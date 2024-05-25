//! Phigros official json chart format

use crate::chart::beat::Beat;
use crate::chart::easing::Easing;
use crate::chart::event::{LineEvent, LineEventKind};
use crate::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};
use crate::format::Format;
use crate::serialization::{LineWrapper, PhiChainChart};
use crate::timing::{BpmList, BpmPoint};
use anyhow::bail;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

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
pub struct OfficialChart {
    offset: f32,
    #[serde(rename(deserialize = "judgeLineList"))]
    lines: Vec<Line>,
}

impl Format for OfficialChart {
    fn into_phichain(self) -> anyhow::Result<PhiChainChart> {
        if self.lines.is_empty() {
            bail!("Expect at least one line");
        }

        let mut phichain = PhiChainChart::new(
            self.offset * 1000.0,
            BpmList::new(vec![BpmPoint::new(Beat::ZERO, self.lines[0].bpm)]),
            vec![],
        );

        for line in self.lines {
            let t: fn(f32) -> Beat = |x| Beat::from(x * 1.875 / 60.0);
            let x: fn(f32) -> f32 = |x| (x - 0.5) * CANVAS_WIDTH;
            let y: fn(f32) -> f32 = |x| (x - 0.5) * CANVAS_HEIGHT;

            let create_note = |above: bool, note: &Note| {
                let kind: crate::chart::note::NoteKind = match note.kind {
                    NoteKind::Tap => crate::chart::note::NoteKind::Tap,
                    NoteKind::Drag => crate::chart::note::NoteKind::Drag,
                    NoteKind::Hold => crate::chart::note::NoteKind::Hold {
                        hold_beat: t(note.hold_time),
                    },
                    NoteKind::Flick => crate::chart::note::NoteKind::Flick,
                };

                crate::chart::note::Note::new(
                    kind,
                    above,
                    t(note.time),
                    note.x / 18.0 * CANVAS_WIDTH,
                    note.speed,
                )
            };

            let move_event_iter = line.move_events.iter().flat_map(|event| {
                vec![
                    LineEvent {
                        kind: LineEventKind::X,
                        start: x(event.start_x),
                        end: x(event.end_x),
                        start_beat: t(event.start_time),
                        end_beat: t(event.end_time),
                        easing: Easing::Linear,
                    },
                    LineEvent {
                        kind: LineEventKind::Y,
                        start: y(event.start_y),
                        end: y(event.end_y),
                        start_beat: t(event.start_time),
                        end_beat: t(event.end_time),
                        easing: Easing::Linear,
                    },
                ]
            });

            let rotate_event_iter = line.rotate_events.iter().map(|event| LineEvent {
                kind: LineEventKind::Rotation,
                start: event.start,
                end: event.end,
                start_beat: t(event.start_time),
                end_beat: t(event.end_time),
                easing: Easing::Linear,
            });

            let opacity_event_iter = line.opacity_events.iter().map(|event| LineEvent {
                kind: LineEventKind::Opacity,
                start: event.start * 255.0,
                end: event.end * 255.0,
                start_beat: t(event.start_time),
                end_beat: t(event.end_time),
                easing: Easing::Linear,
            });

            let speed_event_iter = line.speed_events.iter().map(|event| LineEvent {
                kind: LineEventKind::Speed,
                start: event.value / 2.0 * 9.0,
                end: event.value / 2.0 * 9.0,
                start_beat: t(event.start_time),
                end_beat: t(event.end_time),
                easing: Easing::Linear,
            });

            let mut line = LineWrapper(
                line.notes_above
                    .iter()
                    .map(|x| create_note(true, x))
                    .chain(line.notes_below.iter().map(|x| create_note(false, x)))
                    .collect(),
                move_event_iter
                    .chain(rotate_event_iter)
                    .chain(opacity_event_iter)
                    .chain(speed_event_iter)
                    .collect(),
            );

            let mut speed_events = line
                .1
                .iter()
                .filter(|e| matches!(e.kind, LineEventKind::Speed))
                .collect::<Vec<_>>();
            speed_events.sort_by_key(|e| e.start_beat);

            for note in &mut line.0 {
                if let crate::chart::note::NoteKind::Hold { .. } = note.kind {
                    let mut speed = 0.0;
                    for event in &speed_events {
                        let value = event.evaluate(note.beat.value());
                        if let Some(value) = value {
                            speed = value;
                        }
                    }

                    note.speed /= speed / 9.0 * 2.0;
                }
            }

            phichain.lines.push(line);
        }

        Ok(phichain)
    }

    fn from_phichain(_phichain: PhiChainChart) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        unimplemented!();
    }
}
