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

const EVENT_VALUE_EPSILON: f32 = 1e-4;
const EASING_FITTING_EPSILON: f32 = 1e-1;

const EASING_FITTING_POSSIBLE_EASINGS: [Easing; 31] = [
    Easing::Linear,
    Easing::EaseInSine,
    Easing::EaseOutSine,
    Easing::EaseInOutSine,
    Easing::EaseInQuad,
    Easing::EaseOutQuad,
    Easing::EaseInOutQuad,
    Easing::EaseInCubic,
    Easing::EaseOutCubic,
    Easing::EaseInOutCubic,
    Easing::EaseInQuart,
    Easing::EaseOutQuart,
    Easing::EaseInOutQuart,
    Easing::EaseInQuint,
    Easing::EaseOutQuint,
    Easing::EaseInOutQuint,
    Easing::EaseInExpo,
    Easing::EaseOutExpo,
    Easing::EaseInOutExpo,
    Easing::EaseInCirc,
    Easing::EaseOutCirc,
    Easing::EaseInOutCirc,
    Easing::EaseInBack,
    Easing::EaseOutBack,
    Easing::EaseInOutBack,
    Easing::EaseInElastic,
    Easing::EaseOutElastic,
    Easing::EaseInOutElastic,
    Easing::EaseInBounce,
    Easing::EaseOutBounce,
    Easing::EaseInOutBounce,
];

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

        fn merge_constant_move_events(
            events: Vec<primitive::event::LineEvent>,
        ) -> Vec<primitive::event::LineEvent> {
            let mut merged = Vec::with_capacity(events.len());
            let mut last_x: Option<usize> = None;
            let mut last_y: Option<usize> = None;

            for event in events {
                let target_index = match event.kind {
                    LineEventKind::X => &mut last_x,
                    LineEventKind::Y => &mut last_y,
                    _ => {
                        merged.push(event);
                        continue;
                    }
                };

                if let Some(idx) = *target_index {
                    if let Some(last) = merged.get_mut(idx) {
                        if last.start == last.end
                            && event.start == event.end
                            && last.end == event.start
                            && last.end_beat == event.start_beat
                        {
                            last.end_beat = event.end_beat;
                            continue;
                        }
                    }
                }

                let idx = merged.len();
                *target_index = Some(idx);
                merged.push(event);
            }

            merged
        }

        fn flush_buffer(
            buffer: &mut Vec<primitive::event::LineEvent>,
            fitted_events: &mut Vec<primitive::event::LineEvent>,
            success: &mut i32,
            failed: &mut i32,
        ) {
            if buffer.len() == 1 {
                fitted_events.push(buffer[0])
            } else if buffer.len() > 1 {
                let mut fitted = false;

                let first = *buffer.first().unwrap();
                let last = *buffer.last().unwrap();

                'easing: for easing in EASING_FITTING_POSSIBLE_EASINGS {
                    let target_event = crate::event::LineEvent {
                        kind: first.kind,
                        start_beat: first.start_beat,
                        end_beat: last.end_beat,
                        value: crate::event::LineEventValue::transition(
                            first.start,
                            last.end,
                            easing,
                        ),
                    };

                    for event in buffer.iter() {
                        let expected_start = target_event
                            .evaluate(event.start_beat.value())
                            .value()
                            .unwrap();
                        let expected_end = target_event
                            .evaluate(event.end_beat.value())
                            .value()
                            .unwrap();

                        if (expected_start - event.start).abs() > EASING_FITTING_EPSILON
                            || (expected_end - event.end).abs() > EASING_FITTING_EPSILON
                        {
                            continue 'easing;
                        }
                    }

                    *success += 1;

                    fitted_events.push(primitive::event::LineEvent {
                        kind: first.kind,
                        start_beat: first.start_beat,
                        end_beat: last.end_beat,
                        start: first.start,
                        end: last.end,
                        easing,
                    });

                    fitted = true;
                    break 'easing;
                }

                if !fitted {
                    *failed += 1;
                    fitted_events.append(buffer);
                }
            }
        }

        fn fit_events(
            mut events: Vec<primitive::event::LineEvent>,
            kind: LineEventKind,
        ) -> Vec<primitive::event::LineEvent> {
            if events.is_empty() {
                return vec![];
            }

            events.sort_by_key(|e| e.start_beat);

            let mut fitted_events = vec![];
            let mut buffer: Vec<primitive::event::LineEvent> = vec![];

            let mut expected_duration: Option<Beat> = None;
            let mut is_increasing: Option<bool> = None;

            let mut success = 0;
            let mut failed = 0;

            for event in events.iter().copied() {
                if let Some(last) = buffer.last() {
                    let event_is_increasing = event.end > event.start;
                    let direction_matches =
                        is_increasing.map_or(true, |inc| inc == event_is_increasing);

                    let duration_matches =
                        event.end_beat - event.start_beat == expected_duration.unwrap();

                    if last.end_beat == event.start_beat
                        && (last.end - event.start).abs() <= EASING_FITTING_EPSILON
                        && event.start != event.end
                        && duration_matches
                        && direction_matches
                    {
                        buffer.push(event);
                        is_increasing.get_or_insert(event_is_increasing);
                    } else {
                        flush_buffer(&mut buffer, &mut fitted_events, &mut success, &mut failed);
                        buffer.clear();
                        buffer.push(event);
                        expected_duration.replace(event.end_beat - event.start_beat);
                        is_increasing = None;
                    }
                } else {
                    buffer.push(event);
                    expected_duration.replace(event.end_beat - event.start_beat);
                    is_increasing = None;
                }
            }

            // Flush remaining buffer
            flush_buffer(&mut buffer, &mut fitted_events, &mut success, &mut failed);

            println!(
                "{:?}: success {}, failed {}, success rate {:.2}%",
                kind,
                success,
                failed,
                if success + failed > 0 {
                    (success as f32 / (success + failed) as f32) * 100.0
                } else {
                    0.0
                }
            );

            fitted_events
        }

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

            let move_events = merge_constant_move_events(
                line.move_events
                    .iter()
                    .flat_map(|event| match self.format_version {
                        // reference: https://github.com/MisaLiu/phi-chart-render/blob/master/src/chart/convert/official.js#L203
                        1 => vec![
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
                        ],
                        3 => vec![
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
                        ],
                        _ => unreachable!(),
                    })
                    .collect(),
            );

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

            let events: Vec<_> = move_events
                .into_iter()
                .chain(rotate_event_iter)
                .chain(opacity_event_iter)
                .chain(speed_event_iter)
                .map(|event| {
                    // FIXME: filtering speed events out, seems like current speed evaluation is not correct
                    if event.start == event.end
                        && event.end_beat - event.start_beat > beat!(1, 4)
                        && !event.kind.is_speed()
                    {
                        let mut new_event = event;
                        new_event.end_beat = event.start_beat + beat!(1, 4);

                        new_event
                    } else {
                        event
                    }
                })
                .collect();

            let mut events_by_kind: std::collections::HashMap<LineEventKind, Vec<_>> =
                std::collections::HashMap::new();

            for event in events {
                events_by_kind
                    .entry(event.kind)
                    .or_insert_with(Vec::new)
                    .push(event);
            }

            // Fit events for each kind (except speed)
            let mut fitted_events = vec![];

            for (kind, events) in events_by_kind {
                if kind.is_speed() {
                    // Don't fit speed events, just add them
                    fitted_events.extend(events);
                } else {
                    let mut fitted = fit_events(events, kind);
                    fitted_events.append(&mut fitted);
                }
            }

            // remove redundant constant events suffix
            let mut events_by_kind: std::collections::HashMap<LineEventKind, Vec<_>> =
                std::collections::HashMap::new();

            for event in fitted_events {
                events_by_kind
                    .entry(event.kind)
                    .or_insert_with(Vec::new)
                    .push(event);
            }

            let mut cleaned_events = vec![];

            for (_, mut events) in events_by_kind {
                events.sort_by_key(|e| e.start_beat);

                let mut filtered = vec![];
                for (i, event) in events.iter().enumerate() {
                    if event.start == event.end {
                        if i > 0 && (events[i - 1].end - event.start).abs() < EVENT_VALUE_EPSILON {
                            // skip this redundant suffix constant event
                            continue;
                        }
                    }
                    filtered.push(*event);
                }

                cleaned_events.extend(filtered);
            }

            fitted_events = cleaned_events;

            println!("=========");

            let mut line = primitive::line::Line {
                notes: line
                    .notes_above
                    .iter()
                    .map(|x| create_note(true, x))
                    .chain(line.notes_below.iter().map(|x| create_note(false, x)))
                    .collect(),
                events: fitted_events,
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
