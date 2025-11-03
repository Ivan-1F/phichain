//! Phigros official json chart format

use crate::primitive::PrimitiveChart;
use crate::{primitive, Format};
use phichain_chart::beat;
use phichain_chart::beat::Beat;
use phichain_chart::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};
use phichain_chart::easing::Easing;
use phichain_chart::event::LineEventKind;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr, Debug)]
#[repr(u8)]
pub enum OfficialNoteKind {
    Tap = 1,
    Drag = 2,
    Hold = 3,
    Flick = 4,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OfficialNote {
    #[serde(rename = "type")]
    pub kind: OfficialNoteKind,
    pub time: f32,
    #[serde(rename = "holdTime")]
    pub hold_time: f32,
    #[serde(rename = "positionX")]
    pub x: f32,
    pub speed: f32,

    #[serde(rename = "floorPosition")]
    pub floor_position: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OfficialNumericLineEvent {
    #[serde(rename = "startTime")]
    pub start_time: f32,
    #[serde(rename = "endTime")]
    pub end_time: f32,
    pub start: f32,
    pub end: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OfficialPositionLineEvent {
    #[serde(rename = "startTime")]
    pub start_time: f32,
    #[serde(rename = "endTime")]
    pub end_time: f32,
    #[serde(rename = "start")]
    pub start_x: f32,
    // formatVersion 1 does not have start2
    #[serde(rename = "start2", default)]
    pub start_y: f32,
    #[serde(rename = "end")]
    pub end_x: f32,
    // formatVersion 1 does not have end2
    #[serde(rename = "end2", default)]
    pub end_y: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OfficialSpeedEvent {
    #[serde(rename = "startTime")]
    pub start_time: f32,
    #[serde(rename = "endTime")]
    pub end_time: f32,
    pub value: f32,

    #[serde(rename = "floorPosition", default)]
    pub floor_position: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OfficialLine {
    pub bpm: f32,

    #[serde(rename = "judgeLineMoveEvents")]
    pub move_events: Vec<OfficialPositionLineEvent>,
    #[serde(rename = "judgeLineRotateEvents")]
    pub rotate_events: Vec<OfficialNumericLineEvent>,
    #[serde(rename = "judgeLineDisappearEvents")]
    pub opacity_events: Vec<OfficialNumericLineEvent>,
    #[serde(rename = "speedEvents")]
    pub speed_events: Vec<OfficialSpeedEvent>,

    #[serde(rename = "notesAbove")]
    pub notes_above: Vec<OfficialNote>,
    #[serde(rename = "notesBelow")]
    pub notes_below: Vec<OfficialNote>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OfficialChart {
    #[serde(rename = "formatVersion")]
    pub format_version: u32,
    pub offset: f32,
    #[serde(rename = "judgeLineList")]
    pub lines: Vec<OfficialLine>,
}

impl Format for OfficialChart {
    fn into_primitive(self) -> anyhow::Result<PrimitiveChart> {
        unimplemented!("use official_to_phichain instead")
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
                let start_value = phichain_chart::event::LineEvent::from(event)
                    .evaluate(
                        current_beat.value(),
                        phichain_chart::event::Boundary::Inclusive,
                    )
                    .value()
                    .unwrap();
                let end_value = phichain_chart::event::LineEvent::from(event)
                    .evaluate(
                        current_beat.value() + minimum.value(),
                        phichain_chart::event::Boundary::Inclusive,
                    )
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
            let mut official_line = OfficialLine {
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
                |e| OfficialNumericLineEvent {
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
                |e| OfficialNumericLineEvent {
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
                |e| OfficialSpeedEvent {
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
                    let result = phichain_chart::event::LineEvent::from(*event).evaluate(
                        beat.value(),
                        if start_has_effect {
                            phichain_chart::event::Boundary::Inclusive
                        } else {
                            phichain_chart::event::Boundary::Exclusive
                        },
                    );
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

                official_line.move_events.push(OfficialPositionLineEvent {
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
                    phichain_chart::note::NoteKind::Tap => OfficialNoteKind::Tap,
                    phichain_chart::note::NoteKind::Drag => OfficialNoteKind::Drag,
                    phichain_chart::note::NoteKind::Hold { .. } => OfficialNoteKind::Hold,
                    phichain_chart::note::NoteKind::Flick => OfficialNoteKind::Flick,
                };

                let above = note.above;
                let speed = if matches!(note.kind, phichain_chart::note::NoteKind::Hold { .. }) {
                    let mut speed = 0.0;
                    for event in &speed_events {
                        let result = phichain_chart::event::LineEvent::from(**event).evaluate(
                            note.beat.value(),
                            phichain_chart::event::Boundary::Inclusive,
                        );
                        if let Some(value) = result.value() {
                            speed = value;
                        }
                    }

                    note.speed * (speed / 9.0 * 2.0)
                } else {
                    note.speed
                };

                let note = OfficialNote {
                    kind,
                    time: time(note.beat),
                    hold_time: match note.kind {
                        phichain_chart::note::NoteKind::Hold { hold_beat } => {
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
