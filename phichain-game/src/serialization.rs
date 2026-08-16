use crate::curve_note_track::{CurveNote, CurveNoteTrackReadOnly};
use crate::event::Events;
use crate::line::LineOrder;
use bevy::ecs::system::SystemParam;
use bevy::prelude::{ChildOf, Children, Entity, Query, Res, With, Without};
use phichain_chart::bpm_list::BpmList;
use phichain_chart::event::LineEvent;
use phichain_chart::id::{EventId, Identified, LineId, NoteId};
use phichain_chart::line::Line;
use phichain_chart::note::Note;
use phichain_chart::offset::Offset;
use phichain_chart::serialization::{PhichainChart, SerializedLine};
use std::collections::HashMap;

pub trait SerializeLine {
    /// Serialize a line from the world
    ///
    /// To use this, retrieve `SerializeLineParam` from system params. In case of exclusive-systems (`&mut World`), use `world.run_system_once`
    fn serialize_line(params: &SerializeLineParam, entity: Entity) -> Self;
}

impl SerializeLine for SerializedLine {
    fn serialize_line(params: &SerializeLineParam, entity: Entity) -> Self {
        let children = params.children.get(entity);
        let events = params.events.get(entity);
        let (line, line_id) = params.line.get(entity).expect("Entity is not a line");

        let mut notes = vec![];
        let mut line_events = vec![];
        let mut cnts = vec![];

        let mut note_ids = HashMap::new();

        if let Ok(events) = events {
            for event in events.iter() {
                if let Ok((event, event_id)) = params.line_event.get(*event) {
                    line_events.push(Identified {
                        id: *event_id,
                        data: *event,
                    });
                }
            }
        }

        if let Ok(children) = children {
            for child in children.iter() {
                if let Ok((note, note_id)) = params.note.get(*child) {
                    note_ids.insert(*child, *note_id);
                    notes.push(Identified {
                        id: *note_id,
                        data: *note,
                    });
                }
            }
            for child in children.iter() {
                if let Ok(track) = params.curve_note_track.get(*child) {
                    if let (Some(from), Some(to)) =
                        (note_ids.get(&track.from.0), note_ids.get(&track.to.0))
                    {
                        cnts.push(phichain_chart::curve_note_track::CurveNoteTrack {
                            id: *track.id,
                            from: *from,
                            to: *to,
                            options: track.options.clone(),
                        })
                    }
                }
            }
        }

        let mut child_lines = vec![];

        if let Ok(children) = children {
            for child in children.iter() {
                if params.line.get(*child).is_ok() {
                    child_lines.push(SerializedLine::serialize_line(params, *child));
                }
            }
        }

        SerializedLine::new(
            line.clone(),
            *line_id,
            notes,
            line_events,
            child_lines,
            cnts,
        )
    }
}

#[derive(SystemParam)]
pub struct SerializeLineParam<'w, 's> {
    children: Query<'w, 's, &'static Children>,
    events: Query<'w, 's, &'static Events>,
    line: Query<'w, 's, (&'static Line, &'static LineId)>,

    line_event: Query<'w, 's, (&'static LineEvent, &'static EventId)>,
    note: Query<'w, 's, (&'static Note, &'static NoteId), Without<CurveNote>>,

    curve_note_track: Query<'w, 's, CurveNoteTrackReadOnly>,
}

#[derive(SystemParam)]
pub struct SerializeChartParam<'w, 's> {
    bpm_list: Res<'w, BpmList>,
    offset: Res<'w, Offset>,
    line_query: Query<'w, 's, (Entity, &'static LineOrder), (With<Line>, Without<ChildOf>)>,
}

/// Serialize a chart from the world
///
/// To use this, retrieve `SerializeChartParam` and `SerializeLineParam` from system params. In case of exclusive-systems (`&mut World`), use `world.run_system_once`
pub fn serialize_chart(
    chart_params: SerializeChartParam,
    line_params: SerializeLineParam,
) -> PhichainChart {
    let mut chart =
        PhichainChart::new(chart_params.offset.0, chart_params.bpm_list.clone(), vec![]);

    let lines = chart_params
        .line_query
        .iter()
        .sort::<&LineOrder>()
        .collect::<Vec<_>>();

    for (entity, _) in lines {
        chart
            .lines
            .push(SerializedLine::serialize_line(&line_params, entity));
    }

    chart
}
