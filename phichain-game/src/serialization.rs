use crate::curve_note_track::{CurveNote, CurveNoteTrack};
use crate::event::Events;
use crate::line::LineOrder;
use bevy::prelude::{ChildOf, Children, Entity, With, Without, World};
use phichain_chart::bpm_list::BpmList;
use phichain_chart::event::LineEvent;
use phichain_chart::line::Line;
use phichain_chart::note::Note;
use phichain_chart::offset::Offset;
use phichain_chart::serialization::{PhichainChart, SerializedLine};

pub trait SerializeLine {
    fn serialize_line(world: &World, entity: Entity) -> Self;
}

impl SerializeLine for SerializedLine {
    /// Serialize a line as well as its child lines using a entity from a world
    fn serialize_line(world: &World, entity: Entity) -> Self {
        let children = world.get::<Children>(entity);
        let events = world.get::<Events>(entity);
        let line = world.get::<Line>(entity).expect("Entity is not a line");

        let mut notes: Vec<Note> = vec![];
        let mut line_events: Vec<LineEvent> = vec![];
        let mut cnts = vec![];

        let mut note_entity_order = vec![];

        if let Some(events) = events {
            for event in events.iter() {
                if let Some(event) = world.get::<LineEvent>(*event) {
                    line_events.push(*event);
                }
            }
        }

        if let Some(children) = children {
            for child in children.iter() {
                if let Some(note) = world.get::<Note>(*child) {
                    if world.get::<CurveNote>(*child).is_some() {
                        // skip curve notes
                        continue;
                    }
                    note_entity_order.push(child);
                    notes.push(*note);
                }
            }
            for child in children.iter() {
                if let Some(track) = world.get::<CurveNoteTrack>(*child) {
                    if let Some((from, to)) = track.get_entities() {
                        if let (Some(from), Some(to)) = (
                            note_entity_order.iter().position(|x| **x == from),
                            note_entity_order.iter().position(|x| **x == to),
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

        SerializedLine::new(line.clone(), notes, line_events, child_lines, cnts)
    }
}

pub fn serialize_chart(world: &mut World) -> PhichainChart {
    let bpm_list = world.resource::<BpmList>().clone();
    let offset = world.resource::<Offset>().0;
    let mut chart = PhichainChart::new(offset, bpm_list, vec![]);

    let mut line_query =
        world.query_filtered::<(Entity, &LineOrder), (With<Line>, Without<ChildOf>)>();

    let lines = line_query
        .iter(world)
        .sort::<&LineOrder>()
        .collect::<Vec<_>>();

    for (entity, _) in lines {
        chart
            .lines
            .push(SerializedLine::serialize_line(world, entity));
    }

    chart
}
