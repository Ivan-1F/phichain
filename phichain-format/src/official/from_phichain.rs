use crate::compile::steps::{evaluate_curve_note_tracks, merge_children_line};
use crate::official::schema::{
    OfficialChart, OfficialLine, OfficialNote, OfficialNoteKind, OfficialNumericLineEvent,
    OfficialPositionLineEvent, OfficialSpeedEvent,
};
use phichain_chart::beat;
use phichain_chart::beat::Beat;
use phichain_chart::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};
use phichain_chart::event::{LineEvent, LineEventKind};
use phichain_chart::note::NoteKind;
use phichain_chart::serialization::{PhichainChart, SerializedLine};
use phichain_compiler::helpers::{cut, cut_with_options, fill_gap, CutOptions, EventSequenceError};
use phichain_compiler::sequence::EventSequence;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OfficialOutputError {
    #[error("event sequence error in line '{line_name}' for event kind {event_kind:?}: {source}")]
    EventSequenceError {
        line_name: String,
        event_kind: LineEventKind,
        source: EventSequenceError,
    },
}

pub struct OfficialOutputOptions {
    pub minimum_beat: Beat,
}

impl Default for OfficialOutputOptions {
    fn default() -> Self {
        Self {
            minimum_beat: beat!(1, 32),
        }
    }
}

pub fn phichain_to_official(
    phichain: PhichainChart,
    options: OfficialOutputOptions,
) -> Result<OfficialChart, OfficialOutputError> {
    let phichain = merge_children_line(phichain);
    let phichain = evaluate_curve_note_tracks(phichain);

    let bpm = phichain.bpm_list.0[0].bpm; // take first bpm as base bpm for all lines, normalize all beats using this bpm
    let offset = phichain.offset.0 / 1000.0;

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

        fn process<F, T>(
            line: &SerializedLine,
            kind: LineEventKind,
            mut transform: F,
            target: &mut Vec<T>,
        ) -> Result<(), OfficialOutputError>
        where
            F: FnMut(&LineEvent) -> T,
            T: std::fmt::Debug,
        {
            let events = line
                .events
                .iter()
                .filter(|e| e.kind == kind)
                .copied()
                .collect::<Vec<_>>();

            let filled_gap = fill_gap(&events, 0.0).map_err(|source| {
                OfficialOutputError::EventSequenceError {
                    line_name: line.line.name.clone(),
                    event_kind: kind,
                    source,
                }
            })?;

            for event in filled_gap {
                let event_segments = cut_with_options(
                    event,
                    beat!(1, 32),
                    CutOptions {
                        force_linear: kind.is_speed(),
                    },
                );
                let mut transformed_events = event_segments
                    .iter()
                    .map(&mut transform)
                    .collect::<Vec<_>>();
                target.append(&mut transformed_events);
            }

            Ok(())
        }

        process(
            &line,
            LineEventKind::Rotation,
            |e| OfficialNumericLineEvent {
                start_time: time(e.start_beat),
                end_time: time(e.end_beat),
                start: e.value.start(),
                end: e.value.end(),
            },
            &mut official_line.rotate_events,
        )?;

        process(
            &line,
            LineEventKind::Opacity,
            |e| OfficialNumericLineEvent {
                start_time: time(e.start_beat),
                end_time: time(e.end_beat),
                start: e.value.start() / 255.0,
                end: e.value.end() / 255.0,
            },
            &mut official_line.opacity_events,
        )?;

        process(
            &line,
            LineEventKind::Speed,
            |e| OfficialSpeedEvent {
                start_time: time(e.start_beat),
                end_time: time(e.end_beat),
                value: e.value.start() / 9.0 * 2.0,
                floor_position: 0.0, // this will be calculated later
            },
            &mut official_line.speed_events,
        )?;

        // -------- Move events --------

        let mut x_events = vec![];
        let mut y_events = vec![];

        for event in &line.events {
            match event.kind {
                LineEventKind::X => {
                    let mut events = cut(*event, options.minimum_beat);
                    x_events.append(&mut events);
                }
                LineEventKind::Y => {
                    let mut events = cut(*event, options.minimum_beat);
                    y_events.append(&mut events);
                }
                _ => {}
            }
        }

        x_events.sort_by_key(|e| e.start_beat);
        y_events.sort_by_key(|e| e.start_beat);

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

            let start_x = x_events.evaluate_inclusive(start_beat) / CANVAS_WIDTH + 0.5;
            let end_x = x_events.evaluate_exclusive(end_beat) / CANVAS_WIDTH + 0.5;
            let start_y = y_events.evaluate_inclusive(start_beat) / CANVAS_HEIGHT + 0.5;
            let end_y = y_events.evaluate_exclusive(end_beat) / CANVAS_HEIGHT + 0.5;

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
                NoteKind::Tap => OfficialNoteKind::Tap,
                NoteKind::Drag => OfficialNoteKind::Drag,
                NoteKind::Hold { .. } => OfficialNoteKind::Hold,
                NoteKind::Flick => OfficialNoteKind::Flick,
            };

            let above = note.above;
            let speed = if matches!(note.kind, NoteKind::Hold { .. }) {
                let mut speed = 0.0;
                for event in &speed_events {
                    let result = event.evaluate_inclusive(note.beat.value());
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
                    NoteKind::Hold { hold_beat } => time(note.beat + hold_beat) - time(note.beat),
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
