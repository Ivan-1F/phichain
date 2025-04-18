//! Phigros official json chart format

use crate::beat::Beat;
use crate::bpm_list::BpmList;
use crate::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};
use crate::easing::Easing;
use crate::event::LineEventKind;
use crate::primitive::{Format, PrimitiveChart};
use crate::{beat, primitive};
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
struct NumericLineEvent {
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
    // formatVersion 1 does not have start2
    #[serde(rename = "start2", default)]
    start_y: f32,
    #[serde(rename = "end")]
    end_x: f32,
    // formatVersion 1 does not have end2
    #[serde(rename = "end2", default)]
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
    rotate_events: Vec<NumericLineEvent>,
    #[serde(rename = "judgeLineDisappearEvents")]
    opacity_events: Vec<NumericLineEvent>,
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
    fn into_primitive(self) -> anyhow::Result<PrimitiveChart> {
        if self.lines.is_empty() {
            bail!("Expect at least one line");
        }

        if !matches!(self.format_version, 1 | 3) {
            bail!(
                "Unsupported formatVersion, expected 1 or 3, got: {}",
                self.format_version
            );
        }

        let mut primitive = PrimitiveChart {
            offset: self.offset * 1000.0,
            bpm_list: BpmList::single(self.lines[0].bpm),
            ..Default::default()
        };

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
                // reference: https://github.com/MisaLiu/phi-chart-render/blob/master/src/chart/convert/official.js#L203
                match self.format_version {
                    1 => {
                        vec![
                            primitive::event::LineEvent {
                                kind: LineEventKind::X,
                                start: x((event.start_x / 1e3).round() / 880.0),
                                end: x((event.end_x / 1e3).round() / 880.0),
                                easing: Easing::Linear,
                                start_beat: t(event.start_time),
                                end_beat: t(event.end_time),
                            },
                            primitive::event::LineEvent {
                                kind: LineEventKind::Y,
                                start: y(event.start_x % 1e3 / 530.0),
                                end: y(event.end_x % 1e3 / 530.0),
                                easing: Easing::Linear,
                                start_beat: t(event.start_time),
                                end_beat: t(event.end_time),
                            },
                        ]
                    }
                    3 => {
                        vec![
                            primitive::event::LineEvent {
                                kind: LineEventKind::X,
                                start: x(event.start_x),
                                end: x(event.end_x),
                                easing: Easing::Linear,
                                start_beat: t(event.start_time),
                                end_beat: t(event.end_time),
                            },
                            primitive::event::LineEvent {
                                kind: LineEventKind::Y,
                                start: y(event.start_y),
                                end: y(event.end_y),
                                easing: Easing::Linear,
                                start_beat: t(event.start_time),
                                end_beat: t(event.end_time),
                            },
                        ]
                    }
                    _ => unreachable!(),
                }
            });

            let rotate_event_iter =
                line.rotate_events
                    .iter()
                    .map(|event| primitive::event::LineEvent {
                        kind: LineEventKind::Rotation,
                        start: event.start,
                        end: event.end,
                        easing: Easing::Linear,
                        start_beat: t(event.start_time),
                        end_beat: t(event.end_time),
                    });

            let opacity_event_iter =
                line.opacity_events
                    .iter()
                    .map(|event| primitive::event::LineEvent {
                        kind: LineEventKind::Opacity,
                        start: event.start * 255.0,
                        end: event.end * 255.0,
                        easing: Easing::Linear,
                        start_beat: t(event.start_time),
                        end_beat: t(event.end_time),
                    });

            let speed_event_iter =
                line.speed_events
                    .iter()
                    .map(|event| primitive::event::LineEvent {
                        kind: LineEventKind::Speed,
                        start: event.value / 2.0 * 9.0,
                        end: event.value / 2.0 * 9.0,
                        easing: Easing::Linear,
                        start_beat: t(event.start_time),
                        end_beat: t(event.end_time),
                    });

            let mut line = primitive::line::Line {
                notes: line
                    .notes_above
                    .iter()
                    .map(|x| create_note(true, x))
                    .chain(line.notes_below.iter().map(|x| create_note(false, x)))
                    .collect(),
                events: move_event_iter
                    .chain(rotate_event_iter)
                    .chain(opacity_event_iter)
                    .chain(speed_event_iter)
                    .collect(),
            };

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
                        let result =
                            crate::event::LineEvent::from(**event).evaluate(note.beat.value());
                        if let Some(value) = result.value() {
                            speed = value;
                        }
                    }

                    note.speed /= speed / 9.0 * 2.0;
                }
            }

            primitive.lines.push(line);
        }

        Ok(primitive)
    }

    fn from_primitive(phichain: PrimitiveChart) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        fn cut_event(event: primitive::event::LineEvent) -> Vec<primitive::event::LineEvent> {
            if event.start == event.end {
                return vec![event];
            }
            if matches!(event.easing, Easing::Linear) && !event.kind.is_speed() {
                return vec![event];
            }

            let mut events = vec![];

            let minimum = beat!(1, 32);

            let mut current_beat = event.start_beat;

            while current_beat <= event.end_beat {
                let start_value = crate::event::LineEvent::from(event)
                    .evaluate(current_beat.value())
                    .value()
                    .unwrap();
                let end_value = crate::event::LineEvent::from(event)
                    .evaluate(current_beat.value() + minimum.value())
                    .value()
                    .unwrap();
                events.push(primitive::event::LineEvent {
                    kind: event.kind,
                    start: start_value,
                    end: end_value,
                    easing: Easing::Linear,
                    start_beat: current_beat,
                    end_beat: current_beat + minimum,
                });
                current_beat += minimum;
            }

            events
        }

        let bpm = phichain.bpm_list.0[0].bpm; // take first bpm as base bpm for all lines, normalize all beats using this bpm
        let offset = phichain.offset / 1000.0;

        let time = |beat: Beat| phichain.bpm_list.normalize_beat(bpm, beat).value() * 60.0 / 1.875;

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

            fn connect_events(
                events: &[primitive::event::LineEvent],
            ) -> Vec<primitive::event::LineEvent> {
                let mut events = events.to_owned();
                events.sort_by_key(|e| e.start_beat);

                let mut split_beats = vec![];
                for event in &events {
                    split_beats.push(event.start_beat);
                    split_beats.push(event.end_beat);
                }
                split_beats.push(beat!(1000000000));
                split_beats.sort();
                split_beats.dedup();

                let mut connected_events = vec![];

                for i in 0..split_beats.len() - 1 {
                    let start_beat = split_beats[i];
                    let end_beat = split_beats[i + 1];
                    if start_beat == end_beat {
                        continue;
                    }

                    let start = evaluate(&events, start_beat, true);
                    let end = evaluate(&events, end_beat, false);

                    connected_events.push(primitive::event::LineEvent {
                        kind: events.first().unwrap().kind,
                        start,
                        end,
                        easing: Easing::Linear,
                        start_beat,
                        end_beat,
                    })
                }

                connected_events
            }

            fn process_events<F, T>(
                line: &primitive::line::Line,
                kind: LineEventKind,
                mut transform: F,
                target: &mut Vec<T>,
            ) where
                F: FnMut(&primitive::event::LineEvent) -> T,
                T: std::fmt::Debug,
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
                |e| NumericLineEvent {
                    start_time: time(e.start_beat),
                    end_time: time(e.end_beat),
                    start: e.start,
                    end: e.end,
                },
                &mut official_line.rotate_events,
            );

            process_events(
                &line,
                LineEventKind::Opacity,
                |e| NumericLineEvent {
                    start_time: time(e.start_beat),
                    end_time: time(e.end_beat),
                    start: e.start / 255.0,
                    end: e.end / 255.0,
                },
                &mut official_line.opacity_events,
            );

            process_events(
                &line,
                LineEventKind::Speed,
                |e| SpeedEvent {
                    start_time: time(e.start_beat),
                    end_time: time(e.end_beat),
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

            fn evaluate(
                events: &Vec<primitive::event::LineEvent>,
                beat: Beat,
                start_has_effect: bool,
            ) -> f32 {
                let mut ret = 0.0;
                for event in events {
                    let result = if start_has_effect {
                        crate::event::LineEvent::from(*event).evaluate(beat.value())
                    } else {
                        crate::event::LineEvent::from(*event).evaluate_start_no_effect(beat.value())
                    };
                    if let Some(value) = result.value() {
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
            split_beats.push(beat!(1000000000));
            split_beats.sort();
            split_beats.dedup();

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
                    start_time: time(start_beat),
                    end_time: time(end_beat),
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
                        let result =
                            crate::event::LineEvent::from(**event).evaluate(note.beat.value());
                        if let Some(value) = result.value() {
                            speed = value;
                        }
                    }

                    note.speed * (speed / 9.0 * 2.0)
                } else {
                    note.speed
                };

                let note = Note {
                    kind,
                    time: time(note.beat),
                    hold_time: match note.kind {
                        crate::note::NoteKind::Hold { hold_beat } => {
                            time(note.beat + hold_beat) - time(note.beat)
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
