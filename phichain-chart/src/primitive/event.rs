use serde::{Deserialize, Serialize};

use crate::beat::Beat;
use crate::easing::Easing;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineEventKind {
    X,
    Y,
    Rotation,
    Opacity,
    Speed,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineEventValue {
    Transition {
        start: f32,
        end: f32,
        easing: Easing,
    },
    Constant(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LineEvent {
    pub kind: LineEventKind,
    pub start_beat: Beat,
    pub end_beat: Beat,
    pub value: LineEventValue,
}
