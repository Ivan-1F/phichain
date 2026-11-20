use crate::compile::events_for_export;
use crate::compile::steps::{evaluate_curve_note_tracks, merge_children_line};
use crate::official::schema::{
    OfficialChart, OfficialLine, OfficialNote, OfficialNoteKind, OfficialNumericLineEvent,
    OfficialPositionLineEvent, OfficialSpeedEvent,
};
use crate::official::{OfficialOutputError, OfficialOutputOptions};
use phichain_chart::beat::Beat;
use phichain_chart::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};
use phichain_chart::event::{LineEvent, LineEventKind};
use phichain_chart::note::NoteKind;
use phichain_chart::serialization::PhichainChart;
use phichain_compiler::helpers::{cut, cut_with_options, fill_gap, CutOptions};
use phichain_compiler::sequence::EventSequence;

const FINAL_EVENT_END_TIME: f32 = 1e9;

pub fn phichain_to_official(
    phichain: PhichainChart,
    options: &OfficialOutputOptions,
) -> Result<OfficialChart, OfficialOutputError> {
    let phichain = merge_children_line(phichain);
    let phichain = evaluate_curve_note_tracks(phichain);

    let bpm = phichain
        .bpm_list
        .0
        .first()
        .ok_or(OfficialOutputError::EmptyBpmList)?
        .bpm;
    let offset = phichain.offset.0 / 1000.0;

    let time = |beat: Beat| phichain.bpm_list.normalize_beat(bpm, beat).value() * 60.0 / 1.875;

    let mut chart = OfficialChart {
        format_version: 3,
        offset,
        lines: vec![],
    };

    for line in phichain.lines {
        let events: Vec<_> = events_for_export(&line).collect();
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
            events: &[LineEvent],
            kind: LineEventKind,
            minimum_beat: Beat,
            mut transform: F,
            target: &mut Vec<T>,
        ) where
            F: FnMut(&LineEvent) -> T,
            T: std::fmt::Debug,
        {
            let events = events
                .iter()
                .copied()
                .filter(|e| e.kind == kind)
                .collect::<Vec<_>>();

            for event in fill_gap(&events, kind.default_value()) {
                let event_segments = cut_with_options(
                    event,
                    minimum_beat,
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
        }

        process(
            &events,
            LineEventKind::Rotation,
            options.minimum_beat,
            |e| OfficialNumericLineEvent {
                start_time: time(e.start_beat),
                end_time: time(e.end_beat),
                start: e.value.start(),
                end: e.value.end(),
            },
            &mut official_line.rotate_events,
        );

        process(
            &events,
            LineEventKind::Opacity,
            options.minimum_beat,
            |e| OfficialNumericLineEvent {
                start_time: time(e.start_beat),
                end_time: time(e.end_beat),
                start: e.value.start() / 255.0,
                end: e.value.end() / 255.0,
            },
            &mut official_line.opacity_events,
        );

        process(
            &events,
            LineEventKind::Speed,
            options.minimum_beat,
            |e| OfficialSpeedEvent {
                start_time: time(e.start_beat),
                end_time: time(e.end_beat),
                value: e.value.start() / 9.0 * 2.0,
                floor_position: 0.0, // this will be calculated later
            },
            &mut official_line.speed_events,
        );

        // -------- Move events --------

        let mut x_events = vec![];
        let mut y_events = vec![];

        for event in events.iter().copied() {
            match event.kind {
                LineEventKind::X => {
                    let mut events = cut(event, options.minimum_beat);
                    x_events.append(&mut events);
                }
                LineEventKind::Y => {
                    let mut events = cut(event, options.minimum_beat);
                    y_events.append(&mut events);
                }
                _ => {}
            }
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
        split_beats.sort();
        split_beats.dedup();

        for (i, &start_beat) in split_beats.iter().enumerate() {
            let start_x = x_events.evaluate_inclusive(start_beat) / CANVAS_WIDTH + 0.5;
            let start_y = y_events.evaluate_inclusive(start_beat) / CANVAS_HEIGHT + 0.5;
            let (end_time, end_x, end_y) = split_beats.get(i + 1).map_or(
                (FINAL_EVENT_END_TIME, start_x, start_y),
                |&end_beat| {
                    (
                        time(end_beat),
                        x_events.evaluate_exclusive(end_beat) / CANVAS_WIDTH + 0.5,
                        y_events.evaluate_exclusive(end_beat) / CANVAS_HEIGHT + 0.5,
                    )
                },
            );

            official_line.move_events.push(OfficialPositionLineEvent {
                start_time: time(start_beat),
                end_time,
                start_x,
                start_y,
                end_x,
                end_y,
            });
        }

        // -------- Notes --------

        let speed_events = events
            .into_iter()
            .filter(|e| matches!(e.kind, LineEventKind::Speed))
            .collect::<Vec<_>>();

        let mut notes = line.notes.iter().map(|n| n.data).collect::<Vec<_>>();
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
                let speed = speed_events.evaluate_inclusive(note.beat);

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
                    FINAL_EVENT_END_TIME
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

#[cfg(test)]
mod tests {
    use super::*;
    use phichain_chart::beat;
    use phichain_chart::bpm_list::{BpmList, BpmPoint};

    #[test]
    fn export_empty_line_with_bpm_slowdown() {
        let chart = PhichainChart {
            bpm_list: BpmList::new(vec![
                BpmPoint::new(Beat::ZERO, 180.0),
                BpmPoint::new(beat!(4), 60.0),
            ]),
            ..PhichainChart::default()
        };

        let exported = phichain_to_official(chart, &OfficialOutputOptions::default()).unwrap();
        let middle = &exported.lines[0].move_events;
        assert_eq!(middle.len(), 2);
        assert_eq!((middle[0].start_time, middle[0].end_time), (0.0, 32.0));
        let tail = &middle[1];
        assert_eq!(tail.start_time, middle[0].end_time);
        assert_eq!(tail.end_time, 1e9);
        assert_eq!((tail.start_x, tail.start_y), (0.5, 0.5));
        assert_eq!((tail.end_x, tail.end_y), (0.5, 0.5));
    }
}
