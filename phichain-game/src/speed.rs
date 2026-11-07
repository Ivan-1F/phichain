use crate::event::EventTrack;
use phichain_chart::bpm_list::BpmList;

#[derive(Debug, Clone, Copy)]
struct SpeedSegment {
    start_time: f32,
    end_time: f32,
    start_value: f32,
    end_value: f32,
}

/// The progress of integrating speed over time, segment by segment
#[derive(Debug, Clone, Copy)]
struct Scan {
    area: f32,
    last_time: f32,
    last_speed: f32,
}

impl Default for Scan {
    fn default() -> Self {
        Self {
            area: 0.0,
            last_time: 0.0,
            last_speed: 10.0,
        }
    }
}

impl Scan {
    /// Integrate up to `time` through one segment
    ///
    /// Returns the distance at `time` if it is reached within the segment, or `None` to go on with the next one.
    fn advance(&mut self, event: &SpeedSegment, time: f32) -> Option<f32> {
        if time <= self.last_time {
            return Some(self.finish(time));
        }

        let gap_end = event.start_time.min(time);
        if gap_end > self.last_time {
            self.area += (gap_end - self.last_time) * self.last_speed;
            self.last_time = gap_end;
        }

        if time <= event.start_time {
            return Some(self.finish(time));
        }

        let time_span = event.end_time - event.start_time;
        if time_span <= 0.0 {
            self.last_time = self.last_time.max(event.end_time);
            self.last_speed = event.end_value;
            return None;
        }

        let seg_start = event.start_time.max(self.last_time);
        let seg_end = event.end_time.min(time);
        if seg_end > seg_start {
            let speed_span = event.end_value - event.start_value;
            let start_ratio = (seg_start - event.start_time) / time_span;
            let end_ratio = (seg_end - event.start_time) / time_span;
            let start_speed = event.start_value + start_ratio * speed_span;
            let end_speed = event.start_value + end_ratio * speed_span;

            self.area += (seg_end - seg_start) * (start_speed + end_speed) / 2.0;

            if time <= event.end_time {
                return Some(self.area);
            }
        }

        self.last_time = event.end_time;
        self.last_speed = event.end_value;
        None
    }

    /// Integrate up to `time` after the last segment
    fn finish(&self, time: f32) -> f32 {
        if time > self.last_time {
            self.area + (time - self.last_time) * self.last_speed
        } else {
            self.area
        }
    }

    fn distance(mut self, segments: &[SpeedSegment], time: f32) -> f32 {
        for segment in segments {
            if let Some(distance) = self.advance(segment, time) {
                return distance;
            }
        }
        self.finish(time)
    }
}

/// Distance travelled by a line over time, driven by its speed events
///
/// A scan fully passes every segment ending before its time, so a query resumes from the first segment it stops in.
#[derive(Debug, Default)]
pub struct SpeedTable {
    segments: Vec<SpeedSegment>,
    /// The scan state before each segment, and after the last one
    scans: Vec<Scan>,
    /// The running max of the time each segment stops a scan at
    stops: Vec<f32>,
}

impl SpeedTable {
    pub fn new(track: &EventTrack, bpm_list: &BpmList) -> Self {
        let segments: Vec<_> = track
            .values()
            .map(|event| SpeedSegment {
                start_time: bpm_list.time_at(event.start_beat),
                end_time: bpm_list.time_at(event.end_beat),
                start_value: event.value.start(),
                end_value: event.value.end(),
            })
            .collect();
        Self::from_segments(segments)
    }

    fn from_segments(segments: Vec<SpeedSegment>) -> Self {
        let mut scan = Scan::default();
        let mut scans = Vec::with_capacity(segments.len() + 1);
        let mut stops = Vec::with_capacity(segments.len());
        let mut stop = f32::NEG_INFINITY;
        for segment in &segments {
            scans.push(scan);
            scan.advance(segment, f32::INFINITY);
            stop = stop.max(segment.start_time.max(segment.end_time));
            stops.push(stop);
        }
        scans.push(scan);
        Self {
            segments,
            scans,
            stops,
        }
    }

    pub fn distance_at(&self, time: f32) -> f32 {
        let first = self.stops.partition_point(|stop| *stop < time);
        // an empty table has no scans and falls back to a line without speed events
        let scan = self.scans.get(first).copied().unwrap_or_default();
        scan.distance(&self.segments[first..], time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    #[test]
    fn table_matches_a_full_scan() {
        let mut rng = StdRng::seed_from_u64(0);
        for _ in 0..200 {
            // gaps, overlaps and instant segments, ordered by start time
            let mut start = 0.0;
            let segments: Vec<_> = (0..rng.random_range(0..20))
                .map(|_| {
                    start += rng.random_range(-1.0..3.0_f32).max(0.0);
                    let end = start + rng.random_range(-0.5..4.0_f32).max(0.0);
                    SpeedSegment {
                        start_time: start,
                        end_time: end,
                        start_value: rng.random_range(-20.0..20.0),
                        end_value: rng.random_range(-20.0..20.0),
                    }
                })
                .collect();

            let mut times: Vec<f32> = (0..50)
                .map(|_| rng.random_range(-2.0..start + 6.0))
                .collect();
            times.extend(segments.iter().flat_map(|s| [s.start_time, s.end_time]));

            let table = SpeedTable::from_segments(segments.clone());
            for time in times {
                assert_eq!(
                    table.distance_at(time),
                    Scan::default().distance(&segments, time),
                    "at {time} in {segments:?}"
                );
            }
        }
    }
}
