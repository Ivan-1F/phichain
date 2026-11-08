use itertools::Itertools;
use phichain_chart::beat::Beat;
use phichain_chart::event::{LineEvent, LineEventKind};
use std::collections::HashMap;

pub trait EventSequence: Sized {
    /// The value decided by the last event in the sequence starting at or before the beat
    fn evaluate_inclusive(&self, beat: Beat) -> f32;

    /// The value decided by the last event in the sequence starting before the beat
    fn evaluate_exclusive(&self, beat: Beat) -> f32;

    fn x(&self) -> Self;
    fn y(&self) -> Self;
    fn rotation(&self) -> Self;
    fn opacity(&self) -> Self;
    fn speed(&self) -> Self;

    fn group_by_kind(&self) -> HashMap<LineEventKind, Self>;

    fn sorted(&self) -> Self;
}

impl EventSequence for Vec<LineEvent> {
    fn evaluate_inclusive(&self, beat: Beat) -> f32 {
        self.iter()
            .rev()
            .find(|event| event.start_beat <= beat)
            .map_or(0.0, |event| event.value_at(beat.value()))
    }

    fn evaluate_exclusive(&self, beat: Beat) -> f32 {
        self.iter()
            .rev()
            .find(|event| event.start_beat < beat)
            .map_or(0.0, |event| event.value_at(beat.value()))
    }

    fn x(&self) -> Self {
        self.iter().filter(|x| x.kind.is_x()).copied().collect()
    }

    fn y(&self) -> Self {
        self.iter().filter(|x| x.kind.is_y()).copied().collect()
    }

    fn rotation(&self) -> Self {
        self.iter()
            .filter(|x| x.kind.is_rotation())
            .copied()
            .collect()
    }

    fn opacity(&self) -> Self {
        self.iter()
            .filter(|x| x.kind.is_opacity())
            .copied()
            .collect()
    }

    fn speed(&self) -> Self {
        self.iter().filter(|x| x.kind.is_speed()).copied().collect()
    }

    fn group_by_kind(&self) -> HashMap<LineEventKind, Self> {
        let mut map = HashMap::new();

        for event in self {
            map.entry(event.kind).or_insert_with(Vec::new).push(*event);
        }
        map
    }

    fn sorted(&self) -> Self {
        self.iter()
            .sorted_by_key(|x| x.start_beat)
            .copied()
            .collect()
    }
}
