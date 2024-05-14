use serde::{Deserialize, Serialize};

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
    pub bpm_list: BpmList,
    pub lines: Vec<LineWrapper>,
}

impl PhiChainChart {
    pub fn new(bpm_list: BpmList, lines: Vec<LineWrapper>) -> Self {
        Self { bpm_list, lines }
    }
}

impl Default for PhiChainChart {
    fn default() -> Self {
        Self {
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
                },
                LineEvent {
                    kind: LineEventKind::Y,
                    start: 0.0,
                    end: 0.0,
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                },
                LineEvent {
                    kind: LineEventKind::Rotation,
                    start: 0.0,
                    end: 0.0,
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                },
                LineEvent {
                    kind: LineEventKind::Opacity,
                    start: 1.0,
                    end: 1.0,
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                },
                LineEvent {
                    kind: LineEventKind::Speed,
                    start: 10.0,
                    end: 10.0,
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                },
            ],
        )
    }
}
