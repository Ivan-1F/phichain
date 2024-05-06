use crate::{
    chart::{
        beat::Beat,
        event::{LineEvent, LineEventBundle, LineEventKind},
        line::LineBundle,
        note::{NoteBundle, TimelineNote},
    },
    layer::NOTE_TIME_LINE_LAYER, selection::SelectedLine,
};

use super::Loader;

use bevy::{prelude::*, render::view::RenderLayers};
use serde::{Deserialize, Serialize};
use serde_repr::*;

#[derive(Serialize_repr, Deserialize_repr, Debug)]
#[repr(u8)]
enum NoteKind {
    Tap = 1,
    Drag = 2,
    Hold = 3,
    Flick = 4,
}

#[derive(Serialize, Deserialize, Debug)]
struct Note {
    #[serde(rename(deserialize = "type"))]
    kind: NoteKind,
    time: f32,
    #[serde(rename(deserialize = "holdTime"))]
    hold_time: f32,
    #[serde(rename(deserialize = "positionX"))]
    x: f32,
    speed: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct NumbericLineEvent {
    #[serde(rename(deserialize = "startTime"))]
    start_time: f32,
    #[serde(rename(deserialize = "endTime"))]
    end_time: f32,
    start: f32,
    end: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct PositionLineEvent {
    #[serde(rename(deserialize = "startTime"))]
    start_time: f32,
    #[serde(rename(deserialize = "endTime"))]
    end_time: f32,
    #[serde(rename(deserialize = "start"))]
    start_x: f32,
    #[serde(rename(deserialize = "start2"))]
    start_y: f32,
    #[serde(rename(deserialize = "end"))]
    end_x: f32,
    #[serde(rename(deserialize = "end2"))]
    end_y: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct SpeedEvent {
    #[serde(rename(deserialize = "startTime"))]
    start_time: f32,
    #[serde(rename(deserialize = "endTime"))]
    end_time: f32,
    value: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Line {
    #[serde(rename(deserialize = "judgeLineMoveEvents"))]
    move_events: Vec<PositionLineEvent>,
    #[serde(rename(deserialize = "judgeLineRotateEvents"))]
    rotate_events: Vec<NumbericLineEvent>,
    #[serde(rename(deserialize = "judgeLineDisappearEvents"))]
    opacity_events: Vec<NumbericLineEvent>,
    #[serde(rename(deserialize = "speedEvents"))]
    speed_events: Vec<SpeedEvent>,

    #[serde(rename(deserialize = "notesAbove"))]
    notes_above: Vec<Note>,
    #[serde(rename(deserialize = "notesBelow"))]
    notes_below: Vec<Note>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Chart {
    offset: f32,
    #[serde(rename(deserialize = "judgeLineList"))]
    lines: Vec<Line>,
}

pub struct OfficialLoader;

impl Loader for OfficialLoader {
    fn load(file: std::fs::File, mut commands: Commands) {
        let chart: Chart = serde_json::from_reader(file).expect("Failed to load chart");
        let mut first_line_id: Option<Entity> = None;
        for line in chart.lines.iter() {
            let t: fn(f32) -> Beat = |x| Beat::from(x * 1.875 / 60.0);

            let mut note_ids: Vec<Entity> = vec![];
            let id = commands
                .spawn(LineBundle::new())
                .with_children(|parent| {
                    let mut spawn_note = |above: bool, note: &Note| {
                        let kind: crate::chart::note::NoteKind = match note.kind {
                            NoteKind::Tap => crate::chart::note::NoteKind::Tap,
                            NoteKind::Drag => crate::chart::note::NoteKind::Drag,
                            NoteKind::Hold => crate::chart::note::NoteKind::Hold {
                                hold_beat: t(note.hold_time),
                            },
                            NoteKind::Flick => crate::chart::note::NoteKind::Flick,
                        };

                        let note_id = parent
                            .spawn(NoteBundle::new(crate::chart::note::Note::new(
                                kind,
                                above,
                                t(note.time),
                                note.x / 18.0,
                            )))
                            .id();
                        note_ids.push(note_id);
                    };

                    for note in line.notes_above.iter() {
                        spawn_note(true, note);
                    }
                    for note in line.notes_below.iter() {
                        spawn_note(false, note);
                    }
                })
                .id();

            match first_line_id {
                None => first_line_id = Some(id),
                _ => {}
            }

            for note_id in note_ids {
                commands.spawn((
                    SpriteBundle::default(),
                    TimelineNote(note_id),
                    RenderLayers::layer(NOTE_TIME_LINE_LAYER),
                ));
            }

            for event in line.move_events.iter() {
                commands.spawn(LineEventBundle::new(LineEvent {
                    kind: LineEventKind::X,
                    start: event.start_x - 0.5,
                    end: event.end_x - 0.5,
                    start_beat: t(event.start_time),
                    end_beat: t(event.end_time),
                    line_id: id,
                }));
                commands.spawn(LineEventBundle::new(LineEvent {
                    kind: LineEventKind::Y,
                    start: event.start_y - 0.5,
                    end: event.end_y - 0.5,
                    start_beat: t(event.start_time),
                    end_beat: t(event.end_time),
                    line_id: id,
                }));
            }
            for event in line.rotate_events.iter() {
                commands.spawn(LineEventBundle::new(LineEvent {
                    kind: LineEventKind::Rotation,
                    start: event.start,
                    end: event.end,
                    start_beat: t(event.start_time),
                    end_beat: t(event.end_time),
                    line_id: id,
                }));
            }
            for event in line.opacity_events.iter() {
                commands.spawn(LineEventBundle::new(LineEvent {
                    kind: LineEventKind::Opacity,
                    start: event.start,
                    end: event.end,
                    start_beat: t(event.start_time),
                    end_beat: t(event.end_time),
                    line_id: id,
                }));
            }
            for event in line.speed_events.iter() {
                commands.spawn(LineEventBundle::new(LineEvent {
                    kind: LineEventKind::Speed,
                    start: event.value / 2.0 * 9.0,
                    end: event.value / 2.0 * 9.0,
                    start_beat: t(event.start_time),
                    end_beat: t(event.end_time),
                    line_id: id,
                }));
            }
        }
        commands.insert_resource(SelectedLine(first_line_id.unwrap()));
    }
}
