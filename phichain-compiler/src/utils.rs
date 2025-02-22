use itertools::Itertools;
use nalgebra::{Isometry2, Rotation2, Vector2};
use phichain_chart::beat;
use phichain_chart::beat::Beat;
use phichain_chart::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};
use phichain_chart::event::LineEvent;
use std::ops::Range;

/// Represents the state of a line
#[derive(Debug, Copy, Clone)]
pub struct LineState {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub opacity: f32,
    #[allow(dead_code)]
    pub speed: f32,
}

impl LineState {
    /// Returns if the line is visible in the viewport
    pub fn is_visible(&self) -> bool {
        if self.opacity <= 0.0 {
            false
        } else if self.x >= 0.0
            && self.x <= CANVAS_WIDTH
            && self.y >= 0.0
            && self.y <= CANVAS_HEIGHT
        {
            true
        } else {
            Isometry2::new(
                Vector2::new(self.x, self.y),
                Rotation2::new(self.rotation.to_radians()).angle(),
            );

            true
        }
    }
}

pub trait EventSequence {
    fn evaluate(&self, beat: Beat) -> f32;
    fn evaluate_state(&self, beat: Beat) -> LineState;
    fn evaluate_start_no_effect(&self, beat: Beat) -> f32;

    fn x(&self) -> Self;
    fn y(&self) -> Self;
    fn rotation(&self) -> Self;
    fn opacity(&self) -> Self;
    fn speed(&self) -> Self;
}

impl EventSequence for Vec<LineEvent> {
    fn evaluate(&self, beat: Beat) -> f32 {
        let mut ret = 0.0;

        for event in self {
            let result = event.evaluate(beat.value());
            if let Some(value) = result.value() {
                ret = value;
            }
        }

        ret
    }

    fn evaluate_state(&self, beat: Beat) -> LineState {
        LineState {
            x: self.x().evaluate(beat),
            y: self.y().evaluate(beat),
            rotation: self.rotation().evaluate(beat),
            opacity: self.opacity().evaluate(beat),
            speed: self.speed().evaluate(beat),
        }
    }

    fn evaluate_start_no_effect(&self, beat: Beat) -> f32 {
        let mut ret = 0.0;

        for event in self {
            let result = event.evaluate_start_no_effect(beat.value());
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
}

pub type BeatRange = Range<Beat>;

pub fn event_split_points(events: &Vec<LineEvent>) -> Vec<Beat> {
    let mut splits = vec![];
    for event in events {
        splits.push(event.start_beat);
        splits.push(event.end_beat);
    }

    splits.sort();
    splits.dedup();

    splits
}

pub fn find_ranges<F>(events: &Vec<LineEvent>, predicate: F) -> Vec<BeatRange>
where
    F: Fn(LineState) -> bool,
{
    let mut ranges: Vec<BeatRange> = vec![];
    let splits = event_split_points(events);

    let minimum = beat!(1, 32);

    let mut current_range_start = None;

    for (from, to) in splits.iter().copied().tuple_windows() {
        let mut current = from;
        while current <= to {
            let state = events.evaluate_state(current);
            if predicate(state) {
                if current_range_start.is_none() {
                    current_range_start = Some(current);
                }
            } else if let Some(current_range_start) = current_range_start.take() {
                ranges.push(current_range_start..current);
            }

            current += minimum;
        }
    }

    if let Some(current_range_start) = current_range_start {
        ranges.push(current_range_start..Beat::MAX);
    }

    ranges
}
