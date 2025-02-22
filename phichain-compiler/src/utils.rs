use nalgebra::{Isometry2, Rotation2, Vector2};
use phichain_chart::beat::Beat;
use phichain_chart::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};
use phichain_chart::event::LineEvent;

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
