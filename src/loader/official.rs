use crate::{
    chart::{
        beat::Beat,
        event::{LineEvent, LineEventBundle, LineEventKind},
        line::LineBundle,
        note::NoteBundle,
    },
    constants::{CANVAS_HEIGHT, CANVAS_WIDTH},
    selection::SelectedLine,
    timing::{BpmList, BpmPoint},
};

use super::Loader;

use bevy::prelude::*;
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
    bpm: f32,

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
    fn load(file: std::fs::File, commands: &mut Commands) {
        let chart: Chart = serde_json::from_reader(file).expect("Failed to load chart");

        let first_line = chart
            .lines
            .first()
            .expect("The chart should has at least one line");
        commands.insert_resource(BpmList::new(vec![BpmPoint::new(
            Beat::ZERO,
            first_line.bpm,
        )]));

        let mut first_line_id: Option<Entity> = None;
        for line in chart.lines.iter() {
            let t: fn(f32) -> Beat = |x| Beat::from(x * 1.875 / 60.0);
            let x: fn(f32) -> f32 = |x| (x - 0.5) * CANVAS_WIDTH;
            let y: fn(f32) -> f32 = |x| (x - 0.5) * CANVAS_HEIGHT;

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
                                note.x / 18.0 * CANVAS_WIDTH,
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

                    for event in line.move_events.iter() {
                        parent.spawn(LineEventBundle::new(LineEvent {
                            kind: LineEventKind::X,
                            start: x(event.start_x),
                            end: x(event.end_x),
                            start_beat: t(event.start_time),
                            end_beat: t(event.end_time),
                        }));
                        parent.spawn(LineEventBundle::new(LineEvent {
                            kind: LineEventKind::Y,
                            start: y(event.start_y),
                            end: y(event.end_y),
                            start_beat: t(event.start_time),
                            end_beat: t(event.end_time),
                        }));
                    }
                    for event in line.rotate_events.iter() {
                        parent.spawn(LineEventBundle::new(LineEvent {
                            kind: LineEventKind::Rotation,
                            start: event.start,
                            end: event.end,
                            start_beat: t(event.start_time),
                            end_beat: t(event.end_time),
                        }));
                    }
                    for event in line.opacity_events.iter() {
                        parent.spawn(LineEventBundle::new(LineEvent {
                            kind: LineEventKind::Opacity,
                            start: event.start,
                            end: event.end,
                            start_beat: t(event.start_time),
                            end_beat: t(event.end_time),
                        }));
                    }
                    for event in line.speed_events.iter() {
                        parent.spawn(LineEventBundle::new(LineEvent {
                            kind: LineEventKind::Speed,
                            start: event.value / 2.0 * 9.0,
                            end: event.value / 2.0 * 9.0,
                            start_beat: t(event.start_time),
                            end_beat: t(event.end_time),
                        }));
                    }
                })
                .id();

            if first_line_id.is_none() {
                first_line_id = Some(id)
            }
        }
        commands.insert_resource(SelectedLine(first_line_id.unwrap()));
    }
}
