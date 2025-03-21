use crate::event;
use crate::helpers::{fill_gap_until, max, sorted, EventSequenceError};
use crate::utils::EventSequence;
use phichain_chart::beat;
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::easing::Easing;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
use phichain_chart::note::SerializedNote;
use phichain_chart::serialization::{PhichainChart, SerializedLine};

fn convert_to_y_event(end_y: f32, bpm_list: &BpmList, speed_event: LineEvent) -> Vec<LineEvent> {
    match speed_event.value {
        LineEventValue::Constant(spd) => {
            let time =
                bpm_list.time_at(speed_event.end_beat) - bpm_list.time_at(speed_event.start_beat);
            vec![LineEvent {
                kind: LineEventKind::Y,
                start_beat: speed_event.start_beat,
                end_beat: speed_event.end_beat,
                value: LineEventValue::transition(
                    end_y + spd * time * 120.0,
                    end_y,
                    Easing::Linear,
                ),
            }]
        }
        LineEventValue::Transition { start, end, .. } => {
            if start == end {
                let time = bpm_list.time_at(speed_event.end_beat)
                    - bpm_list.time_at(speed_event.start_beat);
                return vec![LineEvent {
                    kind: LineEventKind::Y,
                    start_beat: speed_event.start_beat,
                    end_beat: speed_event.end_beat,
                    value: LineEventValue::transition(
                        end_y + start * time * 120.0,
                        end_y,
                        Easing::Linear,
                    ),
                }];
            }

            let minimum = beat!(1, 32);
            let mut events = vec![];

            let mut current_y = end_y;

            let mut current_beat = speed_event.end_beat;
            let mut segments = Vec::new();

            while current_beat > speed_event.start_beat {
                let prev_beat = {
                    let temp = current_beat - minimum;
                    if temp < speed_event.start_beat {
                        speed_event.start_beat
                    } else {
                        temp
                    }
                };

                segments.push((prev_beat, current_beat));
                current_beat = prev_beat;
            }

            for (start_beat, end_beat) in segments.into_iter() {
                let start_speed = speed_event.evaluate(start_beat.value()).value().unwrap();
                let end_speed = speed_event.evaluate(end_beat.value()).value().unwrap();

                let avg_speed = (start_speed + end_speed) / 2.0;

                let segment_time = bpm_list.time_at(end_beat) - bpm_list.time_at(start_beat);

                // println!(
                //     "[{}] {:?} -> {:?} ({}s): {}",
                //     current_y, start_beat, end_beat, segment_time, avg_speed
                // );
                // println!("delta: {}", avg_speed * segment_time * 120.0);

                let start_y = current_y + avg_speed * segment_time * 120.0;

                events.push(LineEvent {
                    kind: LineEventKind::Y,
                    start_beat,
                    end_beat,
                    value: LineEventValue::transition(start_y, current_y, Easing::Linear),
                });

                current_y = start_y;
            }

            println!("events: {speed_event:?} {events:?}");
            events
        }
    }
}

/// Generate a Y event sequence up to a specified beat with a given speed event sequence.
///
/// The `end_beat` of the last event will be the same as `until`. The end value of the last event will be 0
///
/// Speed of 1 mean moving 120 units per second. Speed events will only be linear. Top of the canvas is larger.
pub fn create_y_events(
    bpm_list: &BpmList,
    speed_events: Vec<LineEvent>,
    until: Beat,
) -> Result<Vec<LineEvent>, EventSequenceError> {
    dbg!(&speed_events, until);
    if speed_events.is_empty() {
        let duration = bpm_list.time_at(until);
        return Ok(vec![LineEvent {
            kind: LineEventKind::Y,
            start_beat: beat!(0),
            end_beat: until,
            value: LineEventValue::transition(
                bpm_list.time_at(until) + duration * 10.0 * 120.0,
                0.0,
                Easing::Linear,
            ),
        }]);
    }

    let mut speed_events = speed_events.clone();
    speed_events.sort_by_key(|x| x.start_beat);

    // # merge
    //
    // Merge with another event sequence. In the case of overlap, combine the values by summing them
    //
    // |              |-----|      |~~~~~|       |~~~~~~~~~~~~~~~|       |=====|
    // |     |=====|      |-----|     |~~~~~~~~~~~~~~~|
    //
    // |     |=====|  |---|-|---|  |||||||||||||||||||||||||||||||       |=====|

    // 0                                                end
    // v                                                 v
    // v                                                 v
    // v                                                 v
    //
    // |              |==A==|      |==B==|       |=======C=======|       |==D==|
    // |==============|=====|======|=====|=======|===============|=======|=====|      fill the gap
    // |==============|=====|======|=====|=======|=======|                            if C is linear or start == end, cut C until `end`
    // |==============|=====|======|=====|=======|||||||||                            if C is not linear, and start != end, cut to C and trim to `end`

    // speed_events.speed().fill_gap().max(until)

    // fill_gap(&speed_events.speed(), 10.0);

    let mut y_events = vec![];

    let mut current = 0.0;

    let processed_sequence = max(
        &fill_gap_until(&sorted(&speed_events.speed()), until, 10.0)?,
        until,
    )?;

    dbg!(&processed_sequence);

    for event in processed_sequence.iter().rev() {
        let start_time = bpm_list.time_at(event.start_beat);
        let end_time = bpm_list.time_at(event.end_beat);
        let delta = end_time - start_time;

        let mut events = convert_to_y_event(current, bpm_list, *event);

        if let Some(last) = events.last() {
            current = last.value.start();
        }

        y_events.append(&mut events);

        current += delta * 10.0;
    }

    Ok(y_events)
}

pub fn apply_note_level_events(chart: PhichainChart) -> PhichainChart {
    let mut lines = vec![];

    for line in &chart.lines {
        let mut new_line = SerializedLine {
            notes: vec![],
            ..line.clone()
        };

        for note in &line.notes {
            if note.events.is_empty() {
                new_line.notes.push(note.clone());
                continue;
            }

            let mut note_line = SerializedLine {
                // move all events except speed events to line first
                events: note
                    .events
                    .iter()
                    .filter(|x| !x.kind.is_speed())
                    .copied()
                    .collect(),
                ..Default::default()
            };

            // speed is constantly 0
            note_line
                .events
                .push(event!(LineEventKind::Speed, 0 => 1, 0.0));

            // TODO: merge note's Y events
            note_line.events.append(
                &mut create_y_events(
                    &chart.bpm_list,
                    line.events.speed(), // TODO: merge note's speed events
                    note.note.beat,
                )
                .unwrap(), // TODO: handle error
            );

            // push to note to the attached line, removing all the events
            note_line.notes.push(SerializedNote::from_note(note.note));

            dbg!(&note_line);
            new_line.children.push(note_line);
        }

        lines.push(new_line);
    }

    PhichainChart { lines, ..chart }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use phichain_chart::bpm_list::BpmPoint;
//
//     #[test]
//     fn test() {
//         let bpm_list = BpmList::new(vec![BpmPoint::new(beat!(0.0), 176.0)]);
//
//         let y = convert_to_y_event(
//             0.0,
//             bpm_list,
//             LineEvent {
//                 kind: LineEventKind::Speed,
//                 start_beat: beat!(2.0),
//                 end_beat: beat!(5.0),
//                 value: LineEventValue::transition(10.0, 10.0, Easing::Linear),
//                 // value: LineEventValue::transition(10.0, 2.0, Easing::Linear),
//             },
//         );
//
//         // dbg!(&y);
//
//         panic!()
//     }
// }
