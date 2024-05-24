use bevy::hierarchy::Children;
use bevy::prelude::{Entity, With, World};
use serde::{Deserialize, Serialize};

use crate::audio::Offset;
use crate::chart::easing::Easing;
use crate::chart::line::Line;
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

/// A wrapper struct to handle line serialization and deserialization
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl LineWrapper {
    pub fn serialize_line(world: &mut World, entity: Entity) -> Self {
        let mut line_query = world.query_filtered::<&Children, With<Line>>();
        let mut note_query = world.query::<&Note>();
        let mut event_query = world.query::<&LineEvent>();

        let children = line_query.get(world, entity).expect("Entity is not a line");

        let mut notes: Vec<Note> = vec![];
        let mut events: Vec<LineEvent> = vec![];
        for child in children.iter() {
            if let Ok(note) = note_query.get(world, *child) {
                notes.push(*note);
            } else if let Ok(event) = event_query.get(world, *child) {
                events.push(*event);
            }
        }

        LineWrapper(notes, events)
    }
}
