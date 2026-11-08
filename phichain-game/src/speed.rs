use crate::event::EventTrack;
use phichain_chart::bpm_list::BpmList;

/// The speed of a line before its first speed event
const DEFAULT_SPEED: f32 = 10.0;

#[derive(Debug, Clone, Copy)]
struct SpeedSegment {
    start_time: f32,
    end_time: f32,
    start_value: f32,
    end_value: f32,
    /// Distance travelled from time zero when the segment starts
    distance: f32,
}

impl SpeedSegment {
    /// Distance at a time from the start of this segment until the next one starts
    fn distance_at(&self, time: f32) -> f32 {
        if time >= self.end_time {
            let length = self.end_time - self.start_time;
            self.distance
                + length * (self.start_value + self.end_value) / 2.0
                + (time - self.end_time) * self.end_value
        } else {
            let progress = (time - self.start_time) / (self.end_time - self.start_time);
            let speed = self.start_value + (self.end_value - self.start_value) * progress;
            self.distance + (time - self.start_time) * (self.start_value + speed) / 2.0
        }
    }
}

/// Distance travelled by a line over time, driven by its speed events
///
/// Speed changes linearly within an event and holds its end value until the next one starts.
#[derive(Debug, Default)]
pub struct SpeedTable(Vec<SpeedSegment>);

impl SpeedTable {
    pub fn new(track: &EventTrack, bpm_list: &BpmList) -> Self {
        Self::from_segments(track.values().map(|event| {
            [
                bpm_list.time_at(event.start_beat),
                bpm_list.time_at(event.end_beat),
                event.value.start(),
                event.value.end(),
            ]
        }))
    }

    /// Build from `[start_time, end_time, start_value, end_value]` of each event, in order
    fn from_segments(segments: impl IntoIterator<Item = [f32; 4]>) -> Self {
        let mut table: Vec<SpeedSegment> = Vec::new();
        for [start_time, end_time, start_value, end_value] in segments {
            let distance = table.last().map_or(start_time * DEFAULT_SPEED, |previous| {
                previous.distance_at(start_time)
            });
            table.push(SpeedSegment {
                start_time,
                end_time,
                start_value,
                end_value,
                distance,
            });
        }
        Self(table)
    }

    /// Distance travelled from time zero, which stays zero before it
    pub fn distance_at(&self, time: f32) -> f32 {
        let time = time.max(0.0);
        match self.0.partition_point(|segment| segment.start_time <= time) {
            0 => time * DEFAULT_SPEED,
            next => self.0[next - 1].distance_at(time),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    /// Integrate the speed from zero to the time over every piece of constant or linear speed
    fn integrate(segments: &[[f32; 4]], time: f32) -> f32 {
        let mut pieces = vec![];
        let mut held = (f32::NEG_INFINITY, DEFAULT_SPEED);
        for &[start, end, start_value, end_value] in segments {
            pieces.push((held.0, start, held.1, held.1));
            pieces.push((start, end, start_value, end_value));
            held = (end, end_value);
        }
        pieces.push((held.0, f32::INFINITY, held.1, held.1));

        pieces
            .into_iter()
            .map(|(from, to, a, b)| {
                let (low, high) = (from.max(0.0), to.min(time));
                if high <= low {
                    return 0.0;
                }
                let speed = |t: f32| {
                    if a == b {
                        a
                    } else {
                        a + (b - a) * (t - from) / (to - from)
                    }
                };
                (high - low) * (speed(low) + speed(high)) / 2.0
            })
            .sum()
    }

    #[test]
    fn table_matches_integrating_every_piece() {
        let mut rng = StdRng::seed_from_u64(0);
        for _ in 0..200 {
            // events never overlap, with gaps between them
            let mut end = 0.0;
            let segments: Vec<_> = (0..rng.random_range(0..20))
                .map(|_| {
                    let start = end + rng.random_range(-1.0..3.0_f32).max(0.0);
                    end = start + rng.random_range(0.0..4.0_f32);
                    [
                        start,
                        end,
                        rng.random_range(-20.0..20.0),
                        rng.random_range(-20.0..20.0),
                    ]
                })
                .collect();

            let mut times: Vec<f32> = (0..50).map(|_| rng.random_range(-2.0..end + 6.0)).collect();
            times.extend(segments.iter().flat_map(|&[start, end, ..]| [start, end]));

            let table = SpeedTable::from_segments(segments.clone());
            for time in times {
                let expected = integrate(&segments, time);
                let actual = table.distance_at(time);
                assert!(
                    (actual - expected).abs() <= 1e-3 * expected.abs().max(1.0),
                    "{actual} != {expected} at {time} in {segments:?}"
                );
            }
        }
    }
}
