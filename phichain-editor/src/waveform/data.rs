use bevy::prelude::Resource;
use bevy_kira_audio::prelude::{Frame, StaticSoundData};

const BLOCK_SIZE: usize = 64;

/// Extrema include zero and both channels: opposite-phase stereo must not cancel.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Peak {
    pub min: f32,
    pub max: f32,
}

impl Peak {
    fn merge(self, other: Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    fn include(self, frame: Frame) -> Self {
        self.merge(Self {
            min: frame.left.min(frame.right).min(0.0),
            max: frame.left.max(frame.right).max(0.0),
        })
    }
}

#[derive(Resource)]
pub(super) struct Waveform {
    // StaticSoundData shares the decoded samples with playback via Arc.
    sound: StaticSoundData,
    // Level 0 summarizes 64 samples, each subsequent level merges two blocks.
    levels: Vec<Vec<Peak>>,
}

impl Waveform {
    pub fn new(sound: StaticSoundData) -> Self {
        let mut waveform = Self {
            sound,
            levels: Vec::new(),
        };
        let count = waveform.sound.num_frames();
        let first: Vec<_> = (0..count)
            .step_by(BLOCK_SIZE)
            .map(|start| waveform.raw_peak(start, (start + BLOCK_SIZE).min(count)))
            .collect();
        waveform.levels.push(first);
        while waveform.levels.last().unwrap().len() > 1 {
            let next = waveform
                .levels
                .last()
                .unwrap()
                .chunks(2)
                .map(|pair| pair[0].merge(*pair.get(1).unwrap_or(&Peak::default())))
                .collect();
            waveform.levels.push(next);
        }
        waveform
    }

    fn raw_peak(&self, start: usize, end: usize) -> Peak {
        (start..end).fold(Peak::default(), |peak, index| {
            peak.include(self.sound.frame_at_index(index).unwrap())
        })
    }

    /// Exact extrema in [start, end), scanning only the two partial edge blocks.
    fn peak(&self, start: usize, end: usize) -> Peak {
        let head_end = start.next_multiple_of(BLOCK_SIZE).min(end);
        let tail_start = (end / BLOCK_SIZE * BLOCK_SIZE).max(head_end);
        let mut peak = self
            .raw_peak(start, head_end)
            .merge(self.raw_peak(tail_start, end));
        let mut left = head_end / BLOCK_SIZE;
        let right = tail_start / BLOCK_SIZE;
        while left < right {
            let level = (left.trailing_zeros() as usize)
                .min((right - left).ilog2() as usize)
                .min(self.levels.len() - 1);
            peak = peak.merge(self.levels[level][left >> level]);
            left += 1 << level;
        }
        peak
    }

    /// Times are in audio seconds. Intervals outside the audio stay empty rather
    /// than repeating the first/last sample; a partial final block is retained.
    pub fn peaks_between(&self, start: f64, end: f64) -> Option<Peak> {
        if !start.is_finite() || !end.is_finite() || end <= start || self.sound.sample_rate == 0 {
            return None;
        }
        let rate = f64::from(self.sound.sample_rate);
        let count = self.sound.num_frames();
        let first = (start * rate).floor().max(0.0).min(count as f64) as usize;
        let last = (end * rate).ceil().max(0.0).min(count as f64) as usize;
        (first < last).then(|| self.peak(first, last))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sound(frames: Vec<Frame>) -> StaticSoundData {
        StaticSoundData {
            sample_rate: 1024,
            frames: frames.into(),
            settings: Default::default(),
            slice: None,
        }
    }

    #[test]
    fn silence_empty_and_sub_block_audio_are_supported() {
        assert_eq!(Waveform::new(sound(vec![])).peaks_between(0.0, 1.0), None);
        let silent = Waveform::new(sound(vec![Frame::from_mono(0.0); 3]));
        assert_eq!(silent.peaks_between(0.0, 1.0), Some(Peak::default()));
        let single = Waveform::new(sound(vec![Frame::from_mono(0.75)]));
        assert_eq!(
            single.peaks_between(0.0, 1.0),
            Some(Peak {
                min: 0.0,
                max: 0.75
            })
        );
    }

    #[test]
    fn opposite_phase_stereo_and_single_channel_transients_survive() {
        let waveform = Waveform::new(sound(vec![Frame {
            left: 0.8,
            right: -0.8,
        }]));
        assert_eq!(
            waveform.peaks_between(0.0, 1.0),
            Some(Peak {
                min: -0.8,
                max: 0.8
            })
        );
        let waveform = Waveform::new(sound(vec![Frame {
            left: 0.0,
            right: 0.9,
        }]));
        assert_eq!(waveform.peaks_between(0.0, 1.0).unwrap().max, 0.9);
    }

    #[test]
    fn audio_bounds_do_not_repeat_and_partial_tail_is_kept() {
        let mut frames = vec![Frame::from_mono(0.0); 131];
        frames[130] = Frame::from_mono(-0.7);
        let waveform = Waveform::new(sound(frames));
        assert_eq!(waveform.peaks_between(-1.0, 0.0), None);
        assert_eq!(waveform.peaks_between(131.0 / 1024.0, 1.0), None);
        assert_eq!(
            waveform.peaks_between(130.0 / 1024.0, 1.0).unwrap().min,
            -0.7
        );
        assert_eq!(waveform.peaks_between(-1.0, 1.0).unwrap().min, -0.7);
    }

    #[test]
    fn exact_ranges_match_raw_extrema_across_levels_and_block_edges() {
        let frames: Vec<_> = (0..4099)
            .map(|i| Frame {
                left: ((i * 73 % 257) as f32 - 128.0) / 128.0,
                right: ((i * 31 % 199) as f32 - 99.0) / 99.0,
            })
            .collect();
        let waveform = Waveform::new(sound(frames.clone()));
        for start in [0, 1, 63, 64, 65, 127, 128, 1023, 4096] {
            for length in [1, 2, 63, 64, 65, 129, 1024, 4099] {
                let end = (start + length).min(frames.len());
                let samples: Vec<_> = frames[start..end]
                    .iter()
                    .flat_map(|f| [f.left, f.right])
                    .collect();
                let expected = Peak {
                    min: samples.iter().copied().fold(0.0, f32::min),
                    max: samples.iter().copied().fold(0.0, f32::max),
                };
                assert_eq!(waveform.peak(start, end), expected, "{start}..{end}");
            }
        }
    }

    #[test]
    fn impulses_keep_their_time_when_zooming_out() {
        let mut frames = vec![Frame::from_mono(0.0); 4096];
        frames[2048] = Frame::from_mono(1.0);
        let waveform = Waveform::new(sound(frames));
        for width in [1, 4, 64, 512, 1024] {
            assert_eq!(
                waveform
                    .peaks_between((2048 - width) as f64 / 1024.0, 2.0)
                    .unwrap()
                    .max,
                0.0
            );
            assert_eq!(
                waveform
                    .peaks_between(2.0, (2048 + width) as f64 / 1024.0)
                    .unwrap()
                    .max,
                1.0
            );
        }
    }

    #[test]
    fn sliced_audio_uses_the_playback_origin() {
        let mut sound = sound(vec![
            Frame::from_mono(1.0),
            Frame::from_mono(-0.5),
            Frame::from_mono(0.8),
        ]);
        sound.slice = Some((1, 2));
        let waveform = Waveform::new(sound);
        assert_eq!(
            waveform.peaks_between(0.0, 1.0),
            Some(Peak {
                min: -0.5,
                max: 0.0
            })
        );
    }
}
