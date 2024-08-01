use crate::beat::Beat;
use serde::{Deserialize, Serialize};

use crate::bpm_list::BpmList;
use crate::easing::Easing;
use crate::event::{LineEvent, LineEventKind};
use crate::format::Format;
use crate::line::Line;
use crate::migration::CURRENT_FORMAT;
use crate::note::Note;
use crate::offset::Offset;

#[derive(Serialize, Deserialize)]
pub struct PhiChainChart {
    pub format: u64,
    pub offset: Offset,
    pub bpm_list: BpmList,
    pub lines: Vec<LineWrapper>,
}

impl Format for PhiChainChart {
    fn into_phichain(self) -> anyhow::Result<PhiChainChart> {
        Ok(self)
    }

    fn from_phichain(phichain: PhiChainChart) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(phichain)
    }
}

impl PhiChainChart {
    pub fn new(offset: f32, bpm_list: BpmList, lines: Vec<LineWrapper>) -> Self {
        Self {
            format: CURRENT_FORMAT,
            offset: Offset(offset),
            bpm_list,
            lines,
        }
    }
}

impl Default for PhiChainChart {
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
}

impl LineWrapper {
    pub fn new(line: Line, notes: Vec<Note>, events: Vec<LineEvent>) -> Self {
        Self {
            line,
            notes,
            events,
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
        }
    }
}

#[cfg(feature = "bevy")]
impl LineWrapper {
    /// Serialize a line using a entity from a world
    pub fn serialize_line(world: &mut bevy::prelude::World, entity: bevy::prelude::Entity) -> Self {
        use bevy::prelude::*;

        let mut line_query = world.query::<(&Children, &Line)>();
        let mut note_query = world.query::<&Note>();
        let mut event_query = world.query::<&LineEvent>();

        let (children, line) = line_query.get(world, entity).expect("Entity is not a line");

        let mut notes: Vec<Note> = vec![];
        let mut events: Vec<LineEvent> = vec![];
        for child in children.iter() {
            if let Ok(note) = note_query.get(world, *child) {
                notes.push(*note);
            } else if let Ok(event) = event_query.get(world, *child) {
                events.push(*event);
            }
        }

        LineWrapper::new(line.clone(), notes, events)
    }
}
