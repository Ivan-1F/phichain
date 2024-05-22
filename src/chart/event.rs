use crate::chart::easing::{Easing, Tween};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::beat::Beat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineEventKind {
    X,
    Y,
    Rotation,
    Opacity,
    Speed,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LineEvent {
    pub kind: LineEventKind,
    pub start: f32,
    pub end: f32,
    pub start_beat: Beat,
    pub end_beat: Beat,

    pub easing: Easing,
}

impl LineEvent {
    pub fn evaluate(&self, beat: f32) -> Option<f32> {
        let start_beat: f32 = self.start_beat.value();
        let end_beat: f32 = self.end_beat.value();
        if beat >= start_beat && beat <= end_beat {
            let percent = (beat - start_beat) / (end_beat - start_beat);
            return Some(self.start.ease_to(self.end, percent, self.easing));
        } else if beat > end_beat {
            return Some(self.end);
        }

        None
    }

    pub fn duration(&self) -> Beat {
        self.end_beat - self.start_beat
    }
}

#[derive(Bundle)]
pub struct LineEventBundle {
    event: LineEvent,
}

impl LineEventBundle {
    pub fn new(event: LineEvent) -> Self {
        Self { event }
    }
}
