use serde::{Deserialize, Serialize};

use crate::audio::Offset;
use crate::chart::easing::Easing;
use crate::{
    chart::{
        beat::Beat,
        event::{LineEvent, LineEventKind},
        note::Note,
    },
    timing::BpmList,
};

#[derive(Serialize, Deserialize)]
pub struct PhiChainChart {
    pub offset: Offset,
    pub bpm_list: BpmList,
    pub lines: Vec<LineWrapper>,
}

impl PhiChainChart {
    pub fn new(offset: f32, bpm_list: BpmList, lines: Vec<LineWrapper>) -> Self {
        Self {
            offset: Offset(offset),
            bpm_list,
            lines,
        }
    }
}

impl Default for PhiChainChart {
    fn default() -> Self {
        Self {
            offset: Default::default(),
            bpm_list: Default::default(),
            lines: vec![Default::default()],
        }
    }
}

/// A wrapper struct to handle line serialzation and deserialzation
#[derive(Serialize, Deserialize)]
pub struct LineWrapper(pub Vec<Note>, pub Vec<LineEvent>);

/// A default line with no notes and default events
impl Default for LineWrapper {
    fn default() -> Self {
        Self(
            vec![],
            vec![
                LineEvent {
                    kind: LineEventKind::X,
                    start: 0.0,
                    end: 0.0,
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                    easing: Easing::Linear,
                },
                LineEvent {
                    kind: LineEventKind::Y,
                    start: 0.0,
                    end: 0.0,
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                    easing: Easing::Linear,
                },
                LineEvent {
                    kind: LineEventKind::Rotation,
                    start: 0.0,
                    end: 0.0,
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                    easing: Easing::Linear,
                },
                LineEvent {
                    kind: LineEventKind::Opacity,
                    start: 255.0,
                    end: 255.0,
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                    easing: Easing::Linear,
                },
                LineEvent {
                    kind: LineEventKind::Speed,
                    start: 10.0,
                    end: 10.0,
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                    easing: Easing::Linear,
                },
            ],
        )
    }
}
