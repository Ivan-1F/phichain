use crate::beat;
use crate::beat::Beat;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct BpmPoint {
    pub beat: Beat,
    pub bpm: f32,

    #[serde(skip_serializing, default)]
    time: f32,
}

impl PartialEq for BpmPoint {
    fn eq(&self, other: &Self) -> bool {
        self.beat == other.beat && self.bpm == other.bpm
    }
}

impl BpmPoint {
    pub fn new(beat: Beat, bpm: f32) -> Self {
        Self {
            beat,
            bpm,
            time: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Resource))]
pub struct BpmList(pub Vec<BpmPoint>);

impl<'de> Deserialize<'de> for BpmList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let points = Vec::<BpmPoint>::deserialize(deserializer)?;
        let mut bpm_list = BpmList::new(points);
        bpm_list.compute();

        Ok(bpm_list)
    }
}

impl Default for BpmList {
    fn default() -> Self {
        Self::new(vec![BpmPoint::new(Beat::ZERO, 120.0)])
    }
}

impl BpmList {
    pub fn new(points: Vec<BpmPoint>) -> Self {
        let mut list = Self(points);
        list.compute();
        list
    }

    pub fn compute(&mut self) {
        let mut time = 0.0;
        let mut last_beat = 0.0;
        let mut last_bpm = -1.0;
        for point in &mut self.0 {
            if last_bpm != -1.0 {
                time += (point.beat.value() - last_beat) * (60.0 / last_bpm);
            }
            last_beat = point.beat.value();
            last_bpm = point.bpm;
            point.time = time;
        }
    }

    /// Insert a new [`BpmPoint`] into the list
    ///
    /// The point will be inserted in the correct order
    pub fn insert(&mut self, point: BpmPoint) {
        let index = self
            .0
            .iter()
            .position(|p| p.beat.value() > point.beat.value())
            .unwrap_or(self.0.len());
        self.0.insert(index, point);
        self.compute();
    }

    pub fn time_at(&self, beat: Beat) -> f32 {
        let point = self
            .0
            .iter()
            .take_while(|p| p.beat.value() < beat.value())
            .last()
            .or_else(|| self.0.first())
            .expect("No bpm points available");

        point.time + (beat.value() - point.beat.value()) * (60.0 / point.bpm)
    }

    pub fn beat_at(&self, time: f32) -> Beat {
        let point = self
            .0
            .iter()
            .take_while(|p| p.time <= time)
            .last()
            .or_else(|| self.0.first())
            .expect("No bpm points available");

        Beat::from(point.beat.value() + (time - point.time) * point.bpm / 60.0)
    }

    /// Normalize a [`Beat`] on this [`BpmList`] to a [`Beat`] on a fixed BPM
    ///
    /// ```rust
    /// # use phichain_chart::beat;
    /// # use phichain_chart::beat::Beat;
    /// # use phichain_chart::bpm_list::{BpmList, BpmPoint};
    /// let bpm_list = BpmList::new(vec![
    ///     BpmPoint::new(beat!(0), 120.0),
    ///     BpmPoint::new(beat!(4), 240.0),
    /// ]);
    ///
    /// assert_eq!(bpm_list.normalize_beat(120.0, beat!(4)), beat!(4));
    /// assert_eq!(bpm_list.normalize_beat(120.0, beat!(8)), beat!(6));
    /// ```
    pub fn normalize_beat(&self, base: f32, beat: Beat) -> Beat {
        let single_bpm_list = BpmList::new(vec![BpmPoint::new(beat!(0), base)]);
        let time = self.time_at(beat);
        single_bpm_list.beat_at(time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_at() {
        let bpm_list = BpmList::new(vec![
            BpmPoint::new(Beat::ZERO, 120.0),
            BpmPoint::new(Beat::from(4.0), 240.0),
        ]);
        assert_eq!(bpm_list.time_at(Beat::ZERO), 0.0);
        assert_eq!(bpm_list.time_at(Beat::ONE), 0.5);
        assert_eq!(bpm_list.time_at(Beat::from(2.0)), 1.0);
        assert_eq!(bpm_list.time_at(Beat::from(3.0)), 1.5);
        assert_eq!(bpm_list.time_at(Beat::from(4.0)), 2.0);
        assert_eq!(bpm_list.time_at(Beat::from(5.0)), 2.0 + 0.25);
        assert_eq!(bpm_list.time_at(Beat::from(6.0)), 2.0 + 0.25 * 2.0);
        assert_eq!(bpm_list.time_at(Beat::from(7.0)), 2.0 + 0.25 * 3.0);
        assert_eq!(bpm_list.time_at(Beat::from(8.0)), 2.0 + 0.25 * 4.0);
    }

    #[test]
    fn test_beat_at() {
        let bpm_list = BpmList::new(vec![
            BpmPoint::new(Beat::ZERO, 120.0),
            BpmPoint::new(Beat::from(4.0), 240.0),
        ]);
        assert_eq!(bpm_list.beat_at(0.0), Beat::ZERO);
        assert_eq!(bpm_list.beat_at(0.5), Beat::ONE);
        assert_eq!(bpm_list.beat_at(1.0), Beat::from(2.0));
        assert_eq!(bpm_list.beat_at(1.5), Beat::from(3.0));
        assert_eq!(bpm_list.beat_at(2.0), Beat::from(4.0));
        assert_eq!(bpm_list.beat_at(2.0 + 0.25), Beat::from(5.0));
        assert_eq!(bpm_list.beat_at(2.0 + 0.25 * 2.0), Beat::from(6.0));
        assert_eq!(bpm_list.beat_at(2.0 + 0.25 * 3.0), Beat::from(7.0));
        assert_eq!(bpm_list.beat_at(2.0 + 0.25 * 4.0), Beat::from(8.0));
    }

    #[test]
    fn test_bpm_list_serialization() {
        let bpm_list = BpmList::new(vec![
            BpmPoint::new(Beat::ZERO, 120.0),
            BpmPoint::new(Beat::ONE, 240.0),
        ]);

        let string = serde_json::to_string(&bpm_list).unwrap();
        assert_eq!(
            string,
            "[{\"beat\":[0,0,1],\"bpm\":120.0},{\"beat\":[1,0,1],\"bpm\":240.0}]".to_string()
        );

        let deserialized: BpmList = serde_json::from_str(
            "[{\"beat\":[0,0,1],\"bpm\":120.0},{\"beat\":[1,0,1],\"bpm\":240.0}]",
        )
        .unwrap();

        let first = &deserialized.0[0];
        let second = &deserialized.0[1];

        assert_eq!(first.beat, Beat::ZERO);
        assert_eq!(first.bpm, 120.0);
        assert_eq!(first.time, 0.0);

        assert_eq!(second.beat, Beat::ONE);
        assert_eq!(second.bpm, 240.0);
        assert_eq!(second.time, 0.5);
    }

    #[test]
    fn test_normalize_beat() {
        let bpm_list = BpmList::new(vec![
            BpmPoint::new(beat!(0), 120.0),
            BpmPoint::new(beat!(4), 240.0),
        ]);

        assert_eq!(bpm_list.normalize_beat(120.0, beat!(4)), beat!(4));
        assert_eq!(bpm_list.normalize_beat(120.0, beat!(8)), beat!(6));

        assert_eq!(bpm_list.normalize_beat(60.0, beat!(4)), beat!(2));
    }
}
