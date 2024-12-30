use bevy::prelude::{Entity, World};
use phichain_chart::event::LineEvent;
use phichain_chart::line::Line;
use phichain_chart::note::Note;
use phichain_chart::serialization::SerializedLine;
use phichain_game::curve_note_track::{CurveNote, CurveNoteTrack};

pub trait SerializeLine {
    fn serialize_line(world: &World, entity: Entity) -> Self;
}

impl SerializeLine for SerializedLine {
    /// Serialize a line as well as its child lines using a entity from a world
    fn serialize_line(world: &World, entity: Entity) -> Self {
        use bevy::prelude::*;

        let children = world.get::<Children>(entity);
        let line = world.get::<Line>(entity).expect("Entity is not a line");

        let mut notes: Vec<Note> = vec![];
        let mut events: Vec<LineEvent> = vec![];
        let mut cnts = vec![];

        let mut note_entity_order = vec![];

        if let Some(children) = children {
            for child in children.iter() {
                if let Some(note) = world.get::<Note>(*child) {
                    if world.get::<CurveNote>(*child).is_some() {
                        // skip curve notes
                        continue;
                    }
                    note_entity_order.push(*child);
                    notes.push(*note);
                }
                if let Some(event) = world.get::<LineEvent>(*child) {
                    events.push(*event);
                }
            }
            for child in children.iter() {
                if let Some(track) = world.get::<CurveNoteTrack>(*child) {
                    if let Some((from, to)) = track.get_entities() {
                        if let (Some(from), Some(to)) = (
                            note_entity_order.iter().position(|x| *x == from),
                            note_entity_order.iter().position(|x| *x == to),
                        ) {
                            cnts.push(phichain_chart::curve_note_track::CurveNoteTrack {
                                from,
                                to,
                                options: track.options.clone(),
                            })
                        }
                    }
                }
            }
        }

        let mut child_lines = vec![];

        if let Some(children) = children {
            for child in children.iter() {
                if world.get::<Line>(*child).is_some() {
                    child_lines.push(SerializedLine::serialize_line(world, *child));
                }
            }
        }

        SerializedLine::new(line.clone(), notes, events, child_lines, cnts)
    }
}
