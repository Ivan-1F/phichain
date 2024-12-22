use crate::beat::Beat;
use serde::{Deserialize, Serialize};

use crate::bpm_list::BpmList;
use crate::curve_note_track::CurveNoteTrack;
use crate::event::{LineEvent, LineEventKind, LineEventValue};
use crate::line::Line;
use crate::migration::CURRENT_FORMAT;
use crate::note::Note;
use crate::offset::Offset;
use crate::primitive;
use crate::primitive::{Format, PrimitiveChart};

#[derive(Serialize, Deserialize)]
pub struct PhichainChart {
    pub format: u64,
    pub offset: Offset,
    pub bpm_list: BpmList,
    pub lines: Vec<LineWrapper>,
}

impl Format for PhichainChart {
    fn into_primitive(self) -> anyhow::Result<PrimitiveChart> {
        Ok(PrimitiveChart {
            offset: self.offset.0,
            bpm_list: self.bpm_list.clone(),
            lines: self
                .lines
                .iter()
                .map(|line| primitive::line::Line {
                    notes: line.notes.clone(),
                    events: line.events.iter().map(|x| (*x).into()).collect(),
                })
                .collect(),
            ..Default::default()
        })
    }

    fn from_primitive(primitive: PrimitiveChart) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            offset: Offset(primitive.offset),
            bpm_list: primitive.bpm_list,
            lines: primitive
                .lines
                .iter()
                .map(|line| {
                    LineWrapper::new(
                        Line::default(),
                        line.notes.clone(),
                        line.events.iter().map(|x| (*x).into()).collect(),
                        vec![],
                        vec![],
                    )
                })
                .collect(),
            ..Default::default()
        })
    }
}

impl PhichainChart {
    pub fn new(offset: f32, bpm_list: BpmList, lines: Vec<LineWrapper>) -> Self {
        Self {
            format: CURRENT_FORMAT,
            offset: Offset(offset),
            bpm_list,
            lines,
        }
    }
}

impl Default for PhichainChart {
    fn default() -> Self {
        Self {
            format: CURRENT_FORMAT,
            offset: Default::default(),
            bpm_list: Default::default(),
            lines: vec![Default::default()],
        }
    }
}

/// A wrapper struct to handle line serialization and deserialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineWrapper {
    #[serde(flatten)]
    pub line: Line,
    pub notes: Vec<Note>,
    pub events: Vec<LineEvent>,
    pub children: Vec<LineWrapper>,
    pub curve_note_tracks: Vec<CurveNoteTrack>,
}

impl LineWrapper {
    pub fn new(
        line: Line,
        notes: Vec<Note>,
        events: Vec<LineEvent>,
        children: Vec<LineWrapper>,
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
impl Default for LineWrapper {
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

#[cfg(feature = "bevy")]
impl LineWrapper {
    /// Serialize a line as well as its child lines using a entity from a world
    pub fn serialize_line(world: &bevy::prelude::World, entity: bevy::prelude::Entity) -> Self {
        use bevy::prelude::*;

        let children = world.get::<Children>(entity);
        let line = world.get::<Line>(entity).expect("Entity is not a line");

        let mut notes: Vec<Note> = vec![];
        let mut events: Vec<LineEvent> = vec![];
        if let Some(children) = children {
            for child in children.iter() {
                if let Some(note) = world.get::<Note>(*child) {
                    notes.push(*note);
                }
                if let Some(event) = world.get::<LineEvent>(*child) {
                    events.push(*event);
                }
            }
        }

        let mut child_lines = vec![];

        if let Some(children) = children {
            for child in children.iter() {
                if world.get::<Line>(*child).is_some() {
                    child_lines.push(LineWrapper::serialize_line(world, *child));
                }
            }
        }

        LineWrapper::new(line.clone(), notes, events, child_lines, vec![])
    }
}
