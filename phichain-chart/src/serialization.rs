use crate::beat::Beat;
use serde::{Deserialize, Serialize};

use crate::bpm_list::BpmList;
use crate::event::{LineEvent, LineEventKind, LineEventValue};
use crate::format::Format;
use crate::line::Line;
use crate::migration::CURRENT_FORMAT;
use crate::note::{Note, NoteKind};
use crate::offset::Offset;
use crate::primitive;
use crate::primitive::{PrimitiveChart, PrimitiveCompatibleFormat};

#[derive(Serialize, Deserialize)]
pub struct PhichainChart {
    pub format: u64,
    pub offset: Offset,
    pub bpm_list: BpmList,
    pub lines: Vec<LineWrapper>,
}

impl Format for PhichainChart {
    fn into_phichain(self) -> anyhow::Result<PhichainChart> {
        Ok(self)
    }

    fn from_phichain(phichain: PhichainChart) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(phichain)
    }
}

impl PrimitiveCompatibleFormat for PhichainChart {
    fn into_primitive(self) -> anyhow::Result<PrimitiveChart> {
        let mut chart = PrimitiveChart::default();

        chart.offset = self.offset.0;

        chart.bpm_list = primitive::bpm_list::BpmList(
            self.bpm_list
                .0
                .iter()
                .map(|x| primitive::bpm_list::BpmPoint {
                    beat: x.beat,
                    bpm: x.bpm,
                })
                .collect(),
        );

        for LineWrapper { notes, events, .. } in self.lines {
            let mut line = primitive::line::Line::default();

            for note in notes {
                let kind = match note.kind {
                    NoteKind::Tap => primitive::note::NoteKind::Tap,
                    NoteKind::Drag => primitive::note::NoteKind::Drag,
                    NoteKind::Hold { hold_beat } => primitive::note::NoteKind::Hold { hold_beat },
                    NoteKind::Flick => primitive::note::NoteKind::Flick,
                };
                line.notes.push(primitive::note::Note {
                    kind,
                    above: note.above,
                    beat: note.beat,
                    x: note.x,
                    speed: note.speed,
                });
            }

            for event in events {
                let kind = match event.kind {
                    LineEventKind::X => primitive::event::LineEventKind::X,
                    LineEventKind::Y => primitive::event::LineEventKind::Y,
                    LineEventKind::Rotation => primitive::event::LineEventKind::Rotation,
                    LineEventKind::Opacity => primitive::event::LineEventKind::Opacity,
                    LineEventKind::Speed => primitive::event::LineEventKind::Speed,
                };
                let value = match event.value {
                    LineEventValue::Transition { start, end, easing } => {
                        primitive::event::LineEventValue::Transition { start, end, easing }
                    }
                    LineEventValue::Constant(value) => {
                        primitive::event::LineEventValue::Constant(value)
                    }
                };
                line.events.push(primitive::event::LineEvent {
                    kind,
                    start_beat: event.start_beat,
                    end_beat: event.end_beat,
                    value,
                });
            }
        }

        Ok(chart)
    }

    fn from_primitive(phichain: PrimitiveChart) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut chart = Self::default();

        chart.offset = Offset(phichain.offset);

        for primitive::line::Line { notes, events } in phichain.lines {
            let mut line = LineWrapper::default();

            for note in notes {
                let kind = match note.kind {
                    primitive::note::NoteKind::Tap => NoteKind::Tap,
                    primitive::note::NoteKind::Drag => NoteKind::Drag,
                    primitive::note::NoteKind::Hold { hold_beat } => NoteKind::Hold { hold_beat },
                    primitive::note::NoteKind::Flick => NoteKind::Flick,
                };
                line.notes.push(Note {
                    kind,
                    above: note.above,
                    beat: note.beat,
                    x: note.x,
                    speed: note.speed,
                })
            }

            for event in events {
                let kind = match event.kind {
                    primitive::event::LineEventKind::X => LineEventKind::X,
                    primitive::event::LineEventKind::Y => LineEventKind::Y,
                    primitive::event::LineEventKind::Rotation => LineEventKind::Rotation,
                    primitive::event::LineEventKind::Opacity => LineEventKind::Opacity,
                    primitive::event::LineEventKind::Speed => LineEventKind::Speed,
                };
                let value = match event.value {
                    primitive::event::LineEventValue::Transition { start, end, easing } => {
                        LineEventValue::Transition { start, end, easing }
                    }
                    primitive::event::LineEventValue::Constant(value) => {
                        LineEventValue::Constant(value)
                    }
                };
                line.events.push(LineEvent {
                    kind,
                    start_beat: event.start_beat,
                    end_beat: event.end_beat,
                    value,
                });
            }
        }

        Ok(chart)
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
