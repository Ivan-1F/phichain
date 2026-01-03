use itertools::Itertools;
use phichain_chart::beat::Beat;
use phichain_chart::event::{Boundary, LineEvent, LineEventKind};
use std::collections::HashMap;

pub trait EventSequence: Sized {
    fn evaluate(&self, beat: Beat, boundary: Boundary) -> f32;

    fn evaluate_inclusive(&self, beat: Beat) -> f32 {
        self.evaluate(beat, Boundary::Inclusive)
    }

    fn evaluate_exclusive(&self, beat: Beat) -> f32 {
        self.evaluate(beat, Boundary::Exclusive)
    }

    fn x(&self) -> Self;
    fn y(&self) -> Self;
    fn rotation(&self) -> Self;
    fn opacity(&self) -> Self;
    fn speed(&self) -> Self;

    fn group_by_kind(&self) -> HashMap<LineEventKind, Self>;

    fn sorted(&self) -> Self;
}

impl EventSequence for Vec<LineEvent> {
    fn evaluate(&self, beat: Beat, boundary: Boundary) -> f32 {
        let mut ret = 0.0;

        for event in self {
            let result = event.evaluate(beat.value(), boundary);
            if let Some(value) = result.value() {
                ret = value;
            }
        }

        ret
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
