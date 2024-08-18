use crate::beat::Beat;
use crate::easing::Easing;
use crate::event::LineEventKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineEvent {
    pub kind: LineEventKind,
    pub start_beat: Beat,
    pub end_beat: Beat,
    pub start: f32,
    pub end: f32,
    pub easing: Easing,
}
