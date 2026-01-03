use crate::beat::Beat;
use serde::{Deserialize, Serialize};

use crate::bpm_list::BpmList;
use crate::curve_note_track::CurveNoteTrack;
use crate::event::{LineEvent, LineEventKind, LineEventValue};
use crate::line::Line;
use crate::migration::CURRENT_FORMAT;
use crate::note::Note;
use crate::offset::Offset;

#[derive(Serialize, Deserialize)]
pub struct PhichainChart {
    pub format: u64,
    pub offset: Offset,
    pub bpm_list: BpmList,
    pub lines: Vec<SerializedLine>,
}

impl PhichainChart {
    pub fn new(offset: f32, bpm_list: BpmList, lines: Vec<SerializedLine>) -> Self {
        Self {
            format: CURRENT_FORMAT,
            offset: Offset(offset),
            bpm_list,
            lines,
        }
    }
}

impl PhichainChart {
    /// Create an empty [`PhichainChart`] without any lines
    pub fn empty() -> Self {
        Self {
            format: CURRENT_FORMAT,
            offset: Default::default(),
            bpm_list: Default::default(),
            lines: Default::default(),
        }
    }
}

impl Default for PhichainChart {
    fn default() -> Self {
        Self {
            lines: vec![Default::default()],
            ..Self::empty()
        }
    }
}

/// A wrapper struct to handle line serialization and deserialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedLine {
    #[serde(flatten)]
    pub line: Line,
    pub notes: Vec<Note>,
    pub events: Vec<LineEvent>,
    pub children: Vec<SerializedLine>,
    pub curve_note_tracks: Vec<CurveNoteTrack>,
}

impl SerializedLine {
    pub fn new(
        line: Line,
        notes: Vec<Note>,
        events: Vec<LineEvent>,
        children: Vec<SerializedLine>,
        curve_note_tracks: Vec<CurveNoteTrack>,
    ) -> Self {
        Self {
            line,
            notes,
            events,
            children,
            curve_note_tracks,
        }
    }
}

/// A default line with no notes and default events
impl Default for SerializedLine {
    fn default() -> Self {
        Self {
            line: Default::default(),
            notes: Default::default(),
            events: vec![
                LineEvent {
                    kind: LineEventKind::X,
                    value: LineEventValue::constant(0.0),
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                },
                LineEvent {
                    kind: LineEventKind::Y,
                    value: LineEventValue::constant(0.0),
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                },
                LineEvent {
                    kind: LineEventKind::Rotation,
                    value: LineEventValue::constant(0.0),
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                },
                LineEvent {
                    kind: LineEventKind::Opacity,
                    value: LineEventValue::constant(0.0),
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                },
                LineEvent {
                    kind: LineEventKind::Speed,
                    value: LineEventValue::constant(10.0),
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                },
            ],
            children: vec![],
            curve_note_tracks: vec![],
        }
    }
}
