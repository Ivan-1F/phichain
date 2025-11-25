use phichain_chart::beat::Beat;
use phichain_chart::easing::Easing;
use phichain_chart::event::LineEventKind;
use phichain_chart::event::LineEventValue;
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

impl From<phichain_chart::event::LineEvent> for LineEvent {
    fn from(event: phichain_chart::event::LineEvent) -> Self {
        match event.value {
            LineEventValue::Transition { start, end, easing } => Self {
                kind: event.kind,
                start_beat: event.start_beat,
                end_beat: event.end_beat,
                start,
                end,
                easing,
            },
            LineEventValue::Constant(value) => Self {
                kind: event.kind,
                start_beat: event.start_beat,
                end_beat: event.end_beat,
                start: value,
                end: value,
                easing: Easing::Linear,
            },
        }
    }
}

impl From<LineEvent> for phichain_chart::event::LineEvent {
    fn from(event: LineEvent) -> Self {
        Self {
            kind: event.kind,
            start_beat: event.start_beat,
            end_beat: event.end_beat,
            value: LineEventValue::transition(event.start, event.end, event.easing),
        }
    }
}
