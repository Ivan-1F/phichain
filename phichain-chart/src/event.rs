use crate::easing::{Easing, Tween};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};

use crate::beat::Beat;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, IntoPrimitive, TryFromPrimitive,
)]
#[repr(u8)]
pub enum LineEventKind {
    X = 1,
    Y,
    Rotation,
    Opacity,
    Speed,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
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

    pub fn evaluate_start_no_effect(&self, beat: f32) -> Option<f32> {
        let start_beat = self.start_beat.value();
        let end_beat = self.end_beat.value();
        if beat > start_beat && beat <= end_beat {
            let percent = (beat - start_beat) / (end_beat - start_beat);
            return Some(self.start.ease_to(self.end, percent, self.easing));
        } else if beat > end_beat {
            return Some(self.end);
        }

        None
    }
}

#[cfg(feature = "bevy")]
#[derive(bevy::prelude::Bundle)]
pub struct LineEventBundle {
    event: LineEvent,
}

#[cfg(feature = "bevy")]
impl LineEventBundle {
    pub fn new(event: LineEvent) -> Self {
        Self { event }
    }
}
