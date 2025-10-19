use crate::official::schema::{OfficialChart, OfficialNote, OfficialNoteKind};
use anyhow::bail;
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};
use phichain_chart::event::LineEvent;
use phichain_chart::serialization::PhichainChart;
use phichain_chart::{beat, event};
use phichain_compiler::helpers::{are_contiguous, fit_easing, map_if, remove_if};
use phichain_compiler::sequence::EventSequence;

pub mod schema;

const EASING_FITTING_EPSILON: f32 = 1e-1;

fn merge_constant_events(events: Vec<LineEvent>) -> Vec<LineEvent> {
    events.into_iter().fold(Vec::new(), |mut merged, event| {
        if let Some(last) = merged.last_mut() {
            if last.value.is_numeric_constant()
                && event.value.is_numeric_constant()
                && are_contiguous(last, &event)
            {
                // extend the previous event instead of adding a new one
                last.end_beat = event.end_beat;
                return merged;
            }
        }
        merged.push(event);
        merged
    })
}

pub fn official_to_phichain(official: OfficialChart) -> anyhow::Result<PhichainChart> {
    use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
    use phichain_chart::note::{Note, NoteKind};
    use phichain_chart::offset::Offset;
    use phichain_chart::serialization::PhichainChart;
    use phichain_chart::serialization::SerializedLine;

    if official.lines.is_empty() {
        bail!("Expect at least one line");
    }

    if !matches!(official.format_version, 1 | 3) {
        bail!(
            "Unsupported formatVersion, expected 1 or 3, got: {}",
            official.format_version
        );
    }

    let mut phichain = PhichainChart {
        offset: Offset(official.offset * 1000.0),
        bpm_list: BpmList::single(official.lines[0].bpm),
        ..Default::default()
    };

    fn flush_buffer(
        buffer: &mut [LineEvent],
        fitted_events: &mut Vec<LineEvent>,
        success: &mut i32,
        failed: &mut i32,
    ) {
        match fit_easing(buffer, EASING_FITTING_EPSILON) {
            Ok(fitted) => {
                *success += 1;
                fitted_events.push(fitted);
            }
            Err(mut original) => {
                if original.len() > 1 {
                    *failed += 1;
                }
                fitted_events.append(&mut original);
            }
        }
    }

    fn fit_events(events: Vec<LineEvent>, kind: LineEventKind) -> Vec<LineEvent> {
        if events.is_empty() {
            return vec![];
        }

        let mut fitted_events = vec![];
        let mut buffer: Vec<LineEvent> = vec![];

        let mut expected_duration: Option<Beat> = None;
        let mut is_increasing: Option<bool> = None;

        let mut success = 0;
        let mut failed = 0;

        for event in events.sorted().iter().copied() {
            if let Some(last) = buffer.last() {
                let event_is_increasing = event.value.end() > event.value.start();
                let direction_matches = is_increasing.is_none_or(|inc| inc == event_is_increasing);

                let duration_matches =
                    event.end_beat - event.start_beat == expected_duration.unwrap();

                if are_contiguous(last, &event)
                    && !event.value.is_numeric_constant()
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

    for line in official.lines {
        let t: fn(f32) -> Beat = |x| Beat::from(x * 1.875 / 60.0);
        let x: fn(f32) -> f32 = |x| (x - 0.5) * CANVAS_WIDTH;
        let y: fn(f32) -> f32 = |x| (x - 0.5) * CANVAS_HEIGHT;

        let create_note = |above: bool, note: &OfficialNote| {
            let kind: NoteKind = match note.kind {
                OfficialNoteKind::Tap => NoteKind::Tap,
                OfficialNoteKind::Drag => NoteKind::Drag,
                OfficialNoteKind::Hold => NoteKind::Hold {
                    hold_beat: t(note.hold_time),
                },
                OfficialNoteKind::Flick => NoteKind::Flick,
            };

            Note::new(
                kind,
                above,
                t(note.time),
                note.x / 18.0 * CANVAS_WIDTH,
                note.speed,
            )
        };

        let move_events = line
            .move_events
            .iter()
            .flat_map(|event| match official.format_version {
                // reference: https://github.com/MisaLiu/phi-chart-render/blob/master/src/chart/convert/official.js#L203
                1 => vec![
                    event!(
                        LineEventKind::X,
                        t(event.start_time) => t(event.end_time),
                        x((event.start_x / 1e3).round() / 880.0) => x((event.end_x / 1e3).round() / 880.0),
                    ),
                    event!(
                        LineEventKind::Y,
                        t(event.start_time) => t(event.end_time),
                        y(event.start_x % 1e3 / 530.0) => y(event.end_x % 1e3 / 530.0),
                    ),
                ],
                3 => vec![
                    event!(
                        LineEventKind::X,
                        t(event.start_time) => t(event.end_time),
                        x(event.start_x) => x(event.end_x),
                    ),
                    event!(
                        LineEventKind::Y,
                        t(event.start_time) => t(event.end_time),
                        y(event.start_y) => y(event.end_y),
                    ),
                ],
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();

        let move_events = [
            merge_constant_events(move_events.x()),
            merge_constant_events(move_events.y()),
        ]
        .concat();

        let rotate_event_iter = line.rotate_events.iter().map(|event| {
            event!(
                LineEventKind::Rotation,
                t(event.start_time) => t(event.end_time),
                event.start => event.end,
            )
        });

        let opacity_event_iter = line.opacity_events.iter().map(|event| {
            event!(
                LineEventKind::Opacity,
                t(event.start_time) => t(event.end_time),
                event.start * 255.0 => event.end * 255.0,
            )
        });

        let speed_event_iter = line.speed_events.iter().map(|event| {
            event!(
                LineEventKind::Speed,
                t(event.start_time) => t(event.end_time),
                event.value / 2.0 * 9.0,
            )
        });

        let events: Vec<_> = move_events
            .into_iter()
            .chain(rotate_event_iter)
            .chain(opacity_event_iter)
            .chain(speed_event_iter)
            .collect();

        // Shrink constant events to 1/4
        let events = map_if(
            &events,
            |event| {
                event.value.is_numeric_constant()
                    && event.duration() > beat!(1, 4)
                    && !event.kind.is_speed() // FIXME: filtering speed events out, seems like current speed evaluation is not correct
            },
            |mut event| {
                event.end_beat = event.start_beat + beat!(1, 4);
                event
            },
        );

        // Fit events for each kind (speed events are kept as-is, others are fitted)
        let events: Vec<_> = events
            .group_by_kind()
            .into_iter()
            .flat_map(|(kind, events)| {
                if kind.is_speed() {
                    events
                } else {
                    fit_events(events, kind)
                }
            })
            .collect();

        // Remove redundant constant suffix events
        let events: Vec<_> = events
            .group_by_kind()
            .into_iter()
            .flat_map(|(_, events)| {
                let sorted_events = events.sorted();

                let mut prev_end_value = None;

                remove_if(&sorted_events, |event| {
                    let is_redundant = prev_end_value.is_some_and(|prev_end| {
                        event.value.is_numeric_constant() && event.value.start() == prev_end
                    });

                    prev_end_value = Some(event.value.end());
                    is_redundant
                })
            })
            .collect();

        // Convert numeric constant transition events to constant events
        let events = map_if(
            &events,
            |event| event.value.is_numeric_constant() && !event.value.is_constant(), // is numeric constant but not really a constant event
            |event| LineEvent {
                value: LineEventValue::constant(event.value.start()),
                ..event
            },
        );

        println!("=========");

        let mut line = SerializedLine {
            notes: line
                .notes_above
                .iter()
                .map(|x| create_note(true, x))
                .chain(line.notes_below.iter().map(|x| create_note(false, x)))
                .collect(),
            events,

            ..Default::default()
        };

        let speed_events = line.events.speed().sorted();

        for note in &mut line.notes {
            if let NoteKind::Hold { .. } = note.kind {
                let mut speed = 0.0;
                for event in &speed_events {
                    let result = event.evaluate(note.beat.value());
                    if let Some(value) = result.value() {
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
