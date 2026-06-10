use crate::beat::Beat;
use serde::{Deserialize, Serialize};

use crate::bpm_list::BpmList;
use crate::curve_note_track::CurveNoteTrack;
use crate::event::{LineEvent, LineEventKind, LineEventValue};
use crate::line::Line;
use crate::migration::{migrate, CURRENT_FORMAT};
use crate::note::Note;
use crate::offset::Offset;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseChartError {
    #[error("invalid chart: {0}")]
    InvalidChart(#[from] serde_json::Error),
    #[error("migration failed: {0}")]
    MigrationFailed(anyhow::Error),
}

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

    /// Parse a chart from JSON, migrating it to the latest format if necessary
    ///
    /// Charts already at the latest format are deserialized directly,
    /// skipping the costly [`serde_json::Value`] intermediate representation
    pub fn from_json_str(json: &str) -> Result<Self, ParseChartError> {
        #[derive(Deserialize)]
        struct FormatProbe {
            #[serde(default)]
            format: u64,
        }

        let probe: FormatProbe = serde_json::from_str(json)?;

        if probe.format == CURRENT_FORMAT {
            Ok(serde_json::from_str(json)?)
        } else {
            let value: serde_json::Value = serde_json::from_str(json)?;
            let migrated = migrate(&value).map_err(ParseChartError::MigrationFailed)?;
            Ok(serde_json::from_value(migrated)?)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_json_str_current_format() {
        let json = serde_json::to_string(&PhichainChart::default()).unwrap();
        let chart = PhichainChart::from_json_str(&json).unwrap();
        assert_eq!(chart.format, CURRENT_FORMAT);
        assert_eq!(chart.lines.len(), 1);
    }

    #[test]
    fn test_from_json_str_old_format_migrates() {
        // a minimal v0 chart: lines are [notes, events] tuples without a format field
        let json = r#"{
          "offset": 0.0,
          "bpm_list": [{ "beat": [0, 0, 1], "bpm": 120.0, "time": 0.0 }],
          "lines": [[[], []]]
        }"#;
        let chart = PhichainChart::from_json_str(json).unwrap();
        assert_eq!(chart.format, CURRENT_FORMAT);
        assert_eq!(chart.lines.len(), 1);
    }

    #[test]
    fn test_from_json_str_invalid() {
        assert!(matches!(
            PhichainChart::from_json_str("not json"),
            Err(ParseChartError::InvalidChart(_))
        ));
    }
}
