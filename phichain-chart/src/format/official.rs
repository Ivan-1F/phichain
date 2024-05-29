//! Phigros official json chart format

use crate::beat;
use crate::beat::Beat;
use crate::bpm_list::{BpmList, BpmPoint};
use crate::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};
use crate::easing::Easing;
use crate::event::{LineEvent, LineEventKind};
use crate::format::Format;
use crate::serialization::{LineWrapper, PhiChainChart};
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
    #[serde(rename = "type")]
    kind: NoteKind,
    time: f32,
    #[serde(rename = "holdTime")]
    hold_time: f32,
    #[serde(rename = "positionX")]
    x: f32,
    speed: f32,

    #[serde(rename = "floorPosition")]
    floor_position: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct NumbericLineEvent {
    #[serde(rename = "startTime")]
    start_time: f32,
    #[serde(rename = "endTime")]
    end_time: f32,
    start: f32,
    end: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct PositionLineEvent {
    #[serde(rename = "startTime")]
    start_time: f32,
    #[serde(rename = "endTime")]
    end_time: f32,
    #[serde(rename = "start")]
    start_x: f32,
    #[serde(rename = "start2")]
    start_y: f32,
    #[serde(rename = "end")]
    end_x: f32,
    #[serde(rename = "end2")]
    end_y: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct SpeedEvent {
    #[serde(rename = "startTime")]
    start_time: f32,
    #[serde(rename = "endTime")]
    end_time: f32,
    value: f32,

    #[serde(rename = "floorPosition", default)]
    floor_position: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Line {
    bpm: f32,

    #[serde(rename = "judgeLineMoveEvents")]
    move_events: Vec<PositionLineEvent>,
    #[serde(rename = "judgeLineRotateEvents")]
    rotate_events: Vec<NumbericLineEvent>,
    #[serde(rename = "judgeLineDisappearEvents")]
    opacity_events: Vec<NumbericLineEvent>,
    #[serde(rename = "speedEvents")]
    speed_events: Vec<SpeedEvent>,

    #[serde(rename = "notesAbove")]
    notes_above: Vec<Note>,
    #[serde(rename = "notesBelow")]
    notes_below: Vec<Note>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OfficialChart {
    #[serde(rename = "formatVersion")]
    format_version: u32,
    offset: f32,
    #[serde(rename = "judgeLineList")]
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
                let kind: crate::note::NoteKind = match note.kind {
                    NoteKind::Tap => crate::note::NoteKind::Tap,
                    NoteKind::Drag => crate::note::NoteKind::Drag,
                    NoteKind::Hold => crate::note::NoteKind::Hold {
                        hold_beat: t(note.hold_time),
                    },
                    NoteKind::Flick => crate::note::NoteKind::Flick,
                };

                crate::note::Note::new(
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

            let mut line = LineWrapper::new(
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
                .events
                .iter()
                .filter(|e| matches!(e.kind, LineEventKind::Speed))
                .collect::<Vec<_>>();
            speed_events.sort_by_key(|e| e.start_beat);

            for note in &mut line.notes {
                if let crate::note::NoteKind::Hold { .. } = note.kind {
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

    fn from_phichain(phichain: PhiChainChart) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        fn cut_event(event: LineEvent) -> Vec<LineEvent> {
            if event.easing == Easing::Linear {
                return vec![event];
            }

            let mut events = vec![];

            let minimum = beat!(1, 32);

            let mut current_beat = event.start_beat;

            while current_beat <= event.end_beat {
                let value = event.evaluate(current_beat.value()).unwrap();
                events.push(LineEvent {
                    kind: event.kind,
                    start: value,
                    end: value,
                    start_beat: current_beat,
                    end_beat: current_beat + minimum,
                    easing: Easing::Linear,
                });
                current_beat += minimum;
            }

            events
        }

        let bpm = phichain.bpm_list.0[0].bpm; // TODO: handle multiple bpm
        let offset = phichain.offset.0 / 1000.0;

        let mut chart = OfficialChart {
            format_version: 3,
            offset,
            lines: vec![],
        };

        for line in phichain.lines {
            let mut official_line = Line {
                bpm,
                move_events: vec![],
                rotate_events: vec![],
                opacity_events: vec![],
                speed_events: vec![],
                notes_above: vec![],
                notes_below: vec![],
            };

            // -------- Events --------

            fn connect_events(events: &[LineEvent]) -> Vec<LineEvent> {
                let mut events = events.to_owned();
                events.sort_by_key(|e| e.start_beat);

                let mut split_beats = vec![];
                for event in &events {
                    split_beats.push(event.start_beat);
                    split_beats.push(event.end_beat);
                }
                split_beats.dedup();
                split_beats.sort();

                let mut connected_events = vec![];

                for i in 0..split_beats.len() - 1 {
                    let start_beat = split_beats[i];
                    let end_beat = split_beats[i + 1];
                    if start_beat == end_beat {
                        continue;
                    }

                    let start = evaluate(&events, start_beat, true);
                    let end = evaluate(&events, end_beat, false);

                    connected_events.push(LineEvent {
                        kind: LineEventKind::X, // does not matter
                        start,
                        end,
                        start_beat,
                        end_beat,
                        easing: Easing::Linear,
                    })
                }

                connected_events
            }

            fn process_events<F, T>(
                line: &LineWrapper,
                kind: LineEventKind,
                mut transform: F,
                target: &mut Vec<T>,
            ) where
                F: FnMut(&LineEvent) -> T,
            {
                let events = connect_events(
                    &line
                        .events
                        .iter()
                        .filter(|e| e.kind == kind)
                        .copied()
                        .collect::<Vec<_>>(),
                );

                for event in events {
                    let events = cut_event(event);
                    let mut transformed_events =
                        events.iter().map(&mut transform).collect::<Vec<_>>();
                    target.append(&mut transformed_events);
                }
            }

            process_events(
                &line,
                LineEventKind::Rotation,
                |e| NumbericLineEvent {
                    start_time: e.start_beat.value() * 60.0 / 1.875,
                    end_time: e.end_beat.value() * 60.0 / 1.875,
                    start: e.start,
                    end: e.end,
                },
                &mut official_line.rotate_events,
            );

            process_events(
                &line,
                LineEventKind::Opacity,
                |e| NumbericLineEvent {
                    start_time: e.start_beat.value() * 60.0 / 1.875,
                    end_time: e.end_beat.value() * 60.0 / 1.875,
                    start: e.start / 255.0,
                    end: e.end / 255.0,
                },
                &mut official_line.opacity_events,
            );

            process_events(
                &line,
                LineEventKind::Speed,
                |e| SpeedEvent {
                    start_time: e.start_beat.value() * 60.0 / 1.875,
                    end_time: e.end_beat.value() * 60.0 / 1.875,
                    value: e.start / 9.0 * 2.0,
                    floor_position: 0.0, // this will be calculated later
                },
                &mut official_line.speed_events,
            );

            // -------- Move events --------

            let mut x_events = vec![];
            let mut y_events = vec![];

            for event in &line.events {
                match event.kind {
                    LineEventKind::X => {
                        let mut events = cut_event(*event);
                        x_events.append(&mut events);
                    }
                    LineEventKind::Y => {
                        let mut events = cut_event(*event);
                        y_events.append(&mut events);
                    }
                    _ => {}
                }
            }

            x_events.sort_by_key(|e| e.start_beat);
            y_events.sort_by_key(|e| e.start_beat);

            fn evaluate(events: &Vec<LineEvent>, beat: Beat, start_has_effect: bool) -> f32 {
                let mut ret = 0.0;
                for event in events {
                    let value = if start_has_effect {
                        event.evaluate(beat.value())
                    } else {
                        event.evaluate_start_no_effect(beat.value())
                    };
                    if let Some(value) = value {
                        ret = value;
                    }
                }

                ret
            }

            let mut split_beats = vec![];
            for event in &x_events {
                split_beats.push(event.start_beat);
                split_beats.push(event.end_beat);
            }
            for event in &y_events {
                split_beats.push(event.start_beat);
                split_beats.push(event.end_beat);
            }
            split_beats.dedup();
            split_beats.sort();

            for i in 0..split_beats.len() - 1 {
                let start_beat = split_beats[i];
                let end_beat = split_beats[i + 1];
                if start_beat == end_beat {
                    continue;
                }

                let start_x = evaluate(&x_events, start_beat, true) / CANVAS_WIDTH + 0.5;
                let end_x = evaluate(&x_events, end_beat, false) / CANVAS_WIDTH + 0.5;
                let start_y = evaluate(&y_events, start_beat, true) / CANVAS_HEIGHT + 0.5;
                let end_y = evaluate(&y_events, end_beat, false) / CANVAS_HEIGHT + 0.5;

                official_line.move_events.push(PositionLineEvent {
                    start_time: start_beat.value() * 60.0 / 1.875,
                    end_time: end_beat.value() * 60.0 / 1.875,
                    start_x,
                    start_y,
                    end_x,
                    end_y,
                });
            }

            // -------- Notes --------

            let mut speed_events = line
                .events
                .iter()
                .filter(|e| matches!(e.kind, LineEventKind::Speed))
                .collect::<Vec<_>>();
            speed_events.sort_by_key(|e| e.start_beat);

            let mut notes = line.notes.clone();
            notes.sort_by_key(|n| n.beat);

            for note in notes {
                let kind = match note.kind {
                    crate::note::NoteKind::Tap => NoteKind::Tap,
                    crate::note::NoteKind::Drag => NoteKind::Drag,
                    crate::note::NoteKind::Hold { .. } => NoteKind::Hold,
                    crate::note::NoteKind::Flick => NoteKind::Flick,
                };

                let above = note.above;
                let speed = if matches!(note.kind, crate::note::NoteKind::Hold { .. }) {
                    let mut speed = 0.0;
                    for event in &speed_events {
                        let value = event.evaluate(note.beat.value());
                        if let Some(value) = value {
                            speed = value;
                        }
                    }

                    note.speed * (speed / 9.0 * 2.0)
                } else {
                    note.speed
                };

                let note = Note {
                    kind,
                    time: note.beat.value() * 60.0 / 1.875,
                    hold_time: match note.kind {
                        crate::note::NoteKind::Hold { hold_beat } => {
                            hold_beat.value() * 60.0 / 1.875
                        }
                        _ => 0.0,
                    },
                    x: note.x / CANVAS_WIDTH * 18.0,
                    speed,
                    floor_position: 0.0, // this will be calculated later
                };

                if above {
                    official_line.notes_above.push(note);
                } else {
                    official_line.notes_below.push(note);
                }
            }

            // -------- Floor Position --------

            let mut floor_position: f32 = 0.0;

            let len = official_line.speed_events.len();

            for i in 0..len {
                let (start_time, end_time, event_floor_position) = {
                    let event = &official_line.speed_events[i];
                    let start_time = event.start_time.max(0.0);
                    let end_time = if i < len - 1 {
                        official_line.speed_events[i + 1].start_time
                    } else {
                        1e9__f32
                    };
                    let value = event.value;

                    let floor_pos = floor_position;
                    floor_position += (end_time - start_time) * value / bpm * 1.875;

                    (start_time, end_time, floor_pos)
                };

                let event = &mut official_line.speed_events[i];

                event.start_time = start_time;
                event.end_time = end_time;
                event.floor_position = event_floor_position;
            }

            for note in official_line
                .notes_above
                .iter_mut()
                .chain(official_line.notes_below.iter_mut())
            {
                let mut v1 = 0.0;
                let mut v2 = 0.0;
                let mut v3 = 0.0;

                for event in &official_line.speed_events {
                    if note.time > event.end_time {
                        continue;
                    }
                    if note.time < event.start_time {
                        break;
                    }

                    v1 = event.floor_position;
                    v2 = event.value;
                    v3 = note.time - event.start_time;
                }

                note.floor_position = v1 + v2 * v3 / bpm * 1.875;
            }

            chart.lines.push(official_line);
        }

        Ok(chart)
    }
}
