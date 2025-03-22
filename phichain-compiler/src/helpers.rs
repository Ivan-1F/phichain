//! A set of helper functions to work with event sequences
//!
//! ```text
//! |========|: events
//! |--------|: linear events
//! |~~~~~~~~|: non-linear events
//! ||||||||||: many 1/32 events
//! ```

use crate::utils::EventSequence;
use itertools::Itertools;
use phichain_chart::beat::Beat;
use phichain_chart::easing::Easing;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
use thiserror::Error;
use tracing::{debug, info};

#[macro_export]
macro_rules! beat {
    () => {
        phichain_chart::beat::Beat::new(0, num::Rational32::new(0, 1))
    };
    ($whole:literal) => {
        phichain_chart::beat::Beat::new($whole as i32, num::Rational32::new(0, 1))
    };
    ($num:literal / $den:literal) => {
        phichain_chart::beat::Beat::new(0, num::Rational32::new($num as i32, $den as i32))
    };
    ($whole:literal + $num:literal / $den:literal) => {
        phichain_chart::beat::Beat::new(
            $whole as i32,
            num::Rational32::new($num as i32, $den as i32),
        )
    };
}

#[macro_export]
macro_rules! event {
    ($kind:expr, $from:expr => $to:expr, $start_value:expr => $end_value:expr, $easing:expr) => {
        phichain_chart::event::LineEvent {
            kind: $kind,
            start_beat: beat!($from),
            end_beat: beat!($to),
            value: phichain_chart::event::LineEventValue::transition(
                $start_value as f32,
                $end_value as f32,
                $easing,
            ),
        }
    };
    ($kind:expr, $from:expr => $to:expr, $value:expr) => {
        phichain_chart::event::LineEvent {
            kind: $kind,
            start_beat: beat!($from),
            end_beat: beat!($to),
            value: phichain_chart::event::LineEventValue::constant($value as f32),
        }
    };
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum EventSequenceError {
    #[error("the event sequence has overlap at {0:?}")]
    Overlap(Beat),
    #[error("events in the event sequence do not share a single kind")]
    DifferentKind,
}

pub fn sorted(events: &[LineEvent]) -> Vec<LineEvent> {
    events
        .iter()
        .sorted_by_key(|x| x.start_beat)
        .copied()
        .collect()
}

/// Check if the given event sequence has overlap
///
/// ```text
/// |         |=====|      |=====|       |===============|       |=====|    true
///
/// |   |=====|        |===============|       |=====|                      false
/// |       |======|
///
/// |   |=====|        |===============|       |=====|                      false
/// |                           |=================|
/// ```
pub fn check_overlap(events: &[LineEvent]) -> Result<(), EventSequenceError> {
    match sorted(events)
        .iter()
        .tuple_windows()
        .find(|(a, b)| a.end_beat > b.start_beat)
    {
        None => Ok(()),
        Some((_, b)) => Err(EventSequenceError::Overlap(b.start_beat)),
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum EnsureSameKindResult {
    Empty,
    Kind(LineEventKind),
}

/// Checks if all events in the sequence share the same [`LineEventKind`]
///
/// - If the sequence is empty, it returns `Ok(EnsureSameKindResult::Empty)`
/// - If all events in the sequence have the same kind, it returns `Ok(EnsureSameKindResult::Kind(LineEventKind))`
/// - If the events differ, it returns `Err(EventSequenceError::DifferentKind)`
pub fn ensure_same_kind(events: &[LineEvent]) -> Result<EnsureSameKindResult, EventSequenceError> {
    if events.is_empty() {
        Ok(EnsureSameKindResult::Empty)
    } else {
        let first = events[0].kind;

        if events.iter().skip(1).all(|x| x.kind == first) {
            Ok(EnsureSameKindResult::Kind(first))
        } else {
            Err(EventSequenceError::DifferentKind)
        }
    }
}

/// Fill the gap in the event sequence until a given beat. The given event sequence should be sorted
///
/// ```text
/// 0                                                                     end
/// v                                                                      v
/// |      |=====|      |=====|       |===============|       |=====|
/// |------|=====|------|=====|-------|===============|-------|=====|------|
/// ```
pub fn fill_gap_until(
    events: &[LineEvent],
    until: Beat,
    default: f32,
) -> Result<Vec<LineEvent>, EventSequenceError> {
    let kind = match ensure_same_kind(events)? {
        EnsureSameKindResult::Empty => {
            return Ok(vec![]);
        }
        EnsureSameKindResult::Kind(kind) => kind,
    };

    check_overlap(events)?;

    let mut last_end = beat!(0);
    let mut last_value = default;

    let mut filled = vec![];

    for event in sorted(events) {
        if event.start_beat > last_end {
            filled.push(LineEvent {
                kind,
                start_beat: last_end,
                end_beat: event.start_beat,
                value: LineEventValue::Constant(last_value),
            });
        }
        filled.push(event);

        last_end = event.end_beat;
        last_value = event.value.end();
    }

    if last_end < until {
        filled.push(LineEvent {
            kind,
            start_beat: last_end,
            end_beat: until,
            value: LineEventValue::Constant(last_value),
        });
    }

    Ok(filled)
}

/// Fill the gap in the event sequence
///
/// ```text
/// |              |=====|      |=====|       |===============|       |=====|
/// |--------------|=====|------|=====|-------|===============|-------|=====|
/// ```
#[allow(dead_code)]
pub fn fill_gap(events: &[LineEvent], default: f32) -> Result<Vec<LineEvent>, EventSequenceError> {
    fill_gap_until(
        events,
        events.last().map(|x| x.end_beat).unwrap_or(beat!(0)),
        default,
    )
}

/// Cut the given event to multiple small segments
///
/// For constant events, return without modification
/// In:  |-----------------------| (constant)
/// Out: |-----------------------| (constant)
///
/// For linear events, return without modification
/// In:  |-----------------------| (linear)
/// Out: |-----------------------| (linear)
///
/// For non-linear event with same start and end value, return with an identical constant event
/// Val: 8                       8
/// In:  |~~~~~~~~~~~~~~~~~~~~~~~| (sine)
/// Out: |-----------------------| (constant)
///
/// For non-linear event with different start and end value, cut it in to 1/32 events
/// In:  |~~~~~~~~~~~~~~~~~~~~~~~| (sine)
/// Out: ||||||||||||||||||||||||| (linear)
pub fn cut(event: LineEvent) -> Vec<LineEvent> {
    match event.value {
        LineEventValue::Constant(_) => vec![event],
        LineEventValue::Transition { start, end, easing } => {
            if matches!(easing, Easing::Linear) {
                return vec![event];
            }

            if start == end {
                return vec![LineEvent {
                    kind: event.kind,
                    start_beat: event.start_beat,
                    end_beat: event.end_beat,
                    value: LineEventValue::constant(start),
                }];
            }

            let minimum = beat!(1 / 32);

            let mut events = vec![];
            let mut current = event.start_beat;

            while current + minimum <= event.end_beat {
                let start_beat = current;
                let end_beat = current + minimum;

                let start_value = event.evaluate(start_beat.value()).value().unwrap();
                let end_value = event.evaluate(end_beat.value()).value().unwrap();

                events.push(LineEvent {
                    kind: event.kind,
                    start_beat,
                    end_beat,
                    value: LineEventValue::transition(start_value, end_value, Easing::Linear),
                });

                current += minimum;
            }

            events
        }
    }
}

// TODO: add test
/// Clamp the event sequence to a range
///
/// ```text
/// 0            min                       max
/// v             v                         v
/// v             v                         v
/// v             v                         v
///
/// |====|====|-------|=====|=======|~~~~~~~~~~~~~~~|=======|=====|
///               |---|=====|=======|||||||||
///
/// ```
///
/// Depending on type of two segment:
///
/// - for linear, clamp it to a shorter linear event
/// - otherwise cut it first and then clamp
pub fn clamp(
    events: &[LineEvent],
    min: Beat,
    max: Beat,
) -> Result<Vec<LineEvent>, EventSequenceError> {
    check_overlap(events)?;
    let events = sorted(events);

    if events.is_empty() {
        return Ok(vec![]);
    }

    let mut clamped = vec![];

    for mut event in events {
        if event.start_beat < min {
            match event.value {
                LineEventValue::Constant(_) => {
                    event.start_beat = min;
                }
                LineEventValue::Transition { start, end, easing } => {
                    if start == end {
                        event.value = LineEventValue::Constant(start);
                        event.start_beat = min;
                    } else if matches!(easing, Easing::Linear) {
                        let edge = event.evaluate(min.value()).value().unwrap();
                        event.value = LineEventValue::transition(edge, end, Easing::Linear);
                        event.start_beat = min;
                    } else {
                        let segments = cut(event);
                        clamped.extend(segments.iter().filter(|x| x.start_beat >= min));
                        continue; // prevent the original event being pushed to `clamped` again
                    }
                }
            }
        }
        if event.end_beat > max {
            match event.value {
                LineEventValue::Constant(_) => {
                    event.end_beat = max;
                }
                LineEventValue::Transition { start, end, easing } => {
                    if start == end {
                        event.value = LineEventValue::Constant(start);
                        event.end_beat = max;
                    } else if matches!(easing, Easing::Linear) {
                        let edge = event.evaluate(max.value()).value().unwrap();
                        event.value = LineEventValue::transition(start, edge, Easing::Linear);
                        event.end_beat = max;
                    } else {
                        let segments = cut(event);
                        clamped.extend(segments.iter().filter(|x| x.end_beat <= max));
                        continue; // prevent the original event being pushed to `clamped` again
                    }
                }
            }
        }

        if event.start_beat == event.end_beat {
            continue;
        }

        clamped.push(event);
    }

    Ok(clamped)
}

/// [`clamp`] from 0 to [`max`]
pub fn max(events: &[LineEvent], max: Beat) -> Result<Vec<LineEvent>, EventSequenceError> {
    clamp(events, beat!(0), max)
}

/// [`clamp`] from [`min`] to the end_beat of the last event
#[allow(dead_code)]
pub fn min(events: &[LineEvent], min: Beat) -> Result<Vec<LineEvent>, EventSequenceError> {
    clamp(
        events,
        min,
        sorted(events)
            .last()
            .map(|x| x.end_beat)
            .unwrap_or(beat!(0)),
    )
}

/// Find groups of overlapping events from two slices
fn find_overlapping_groups(a: &[LineEvent], b: &[LineEvent]) -> Vec<Vec<LineEvent>> {
    let all_events: Vec<LineEvent> = [a, b].concat();

    if all_events.is_empty() {
        return vec![];
    }

    let sorted_events = sorted(&all_events);
    let mut groups: Vec<Vec<LineEvent>> = vec![];
    let mut current_group: Vec<LineEvent> = vec![];
    let mut latest_end = sorted_events[0].end_beat;

    current_group.push(sorted_events[0]);

    for event in sorted_events.iter().skip(1) {
        if event.start_beat < latest_end {
            current_group.push(*event);
            // 更新当前组的最晚结束时间
            if event.end_beat > latest_end {
                latest_end = event.end_beat;
            }
        } else {
            if !current_group.is_empty() {
                groups.push(current_group);
                current_group = vec![];
            }
            current_group.push(*event);
            latest_end = event.end_beat;
        }
    }

    if !current_group.is_empty() {
        groups.push(current_group);
    }

    groups
}

// TODO: currently all overlapping ranges are cut
/// Merge with another event sequence. In the case of overlap, combine the values by summing them
///
/// Input:
/// |              |-----|      |~~~~~|       |~~~~~~~~~~~~~~~|       |=====|
/// |     |=====|      |-----|     |~~~~~~~~~~~~~~~|
/// Output:
/// |     |=====|  |---|-|---|  |||||||||||||||||||||||||||||||       |=====|
#[allow(dead_code)]
pub fn merge(a: &[LineEvent], b: &[LineEvent]) -> Vec<LineEvent> {
    match (a.is_empty(), b.is_empty()) {
        (true, true) => return vec![],
        (true, false) => return b.to_vec(),
        (false, true) => return a.to_vec(),
        _ => {}
    }

    let a = sorted(a);
    let b = sorted(b);

    let overlapping_groups = find_overlapping_groups(&a, &b);

    let mut merged = vec![];

    for group in overlapping_groups {
        if group.len() == 1 {
            debug!("Skipping non-overlapping group: {:?}", group);
            merged.push(group[0]);
            continue;
        }

        let group = sorted(&group);

        // TODO: optimize for linear/constant/same-start-end events. do not always cut

        let start = group
            .iter()
            .min_by_key(|x| x.start_beat)
            .unwrap()
            .start_beat;
        let end = group.iter().max_by_key(|x| x.end_beat).unwrap().end_beat;

        debug!("Merging group: {:?}", group);
        debug!("Overlapping range: {:?} ~ {:?}", start, end);

        let mut current = start;
        let minimum = beat!(1 / 32);

        while current + minimum <= end {
            let start_beat = current;
            let end_beat = current + minimum;

            info!(
                "[{:?}] A: {}, B: {}",
                start_beat,
                a.evaluate(start_beat),
                b.evaluate(start_beat)
            );

            let start_value = a.to_vec().evaluate(start_beat) + b.evaluate(start_beat);
            let end_value = a.to_vec().evaluate(end_beat) + b.evaluate(end_beat);

            merged.push(LineEvent {
                kind: group[0].kind,
                start_beat,
                end_beat,
                value: LineEventValue::transition(start_value, end_value, Easing::Linear),
            });

            current += minimum;
        }
    }

    merged
}

#[cfg(test)]
mod tests {
    use crate::helpers::{
        check_overlap, cut, ensure_same_kind, fill_gap, fill_gap_until, sorted, EventSequenceError,
    };
    use phichain_chart::easing::Easing;
    use phichain_chart::event::LineEventKind;

    #[test]
    fn test_sorted() {
        let unsorted = vec![
            event!(LineEventKind::X, 20 => 30, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 8 => 10, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 5 => 6, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 0 => 1, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 3 => 4, 10.0 => 20.0, Easing::Linear),
        ];

        assert_eq!(
            sorted(&unsorted),
            vec![
                event!(LineEventKind::X, 0 => 1, 10.0 => 20.0, Easing::Linear),
                event!(LineEventKind::X, 3 => 4, 10.0 => 20.0, Easing::Linear),
                event!(LineEventKind::X, 5 => 6, 10.0 => 20.0, Easing::Linear),
                event!(LineEventKind::X, 8 => 10, 10.0 => 20.0, Easing::Linear),
                event!(LineEventKind::X, 20 => 30, 10.0 => 20.0, Easing::Linear),
            ]
        )
    }

    #[test]
    fn test_check_overlap_ok() {
        let events = vec![
            event!(LineEventKind::X, 0 => 1, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 3 => 4, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 5 => 6, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 8 => 10, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 20 => 30, 10.0 => 20.0, Easing::Linear),
        ];

        assert!(check_overlap(&events).is_ok());
    }

    #[test]
    fn test_check_overlap_err() {
        let events = vec![
            event!(LineEventKind::X, 0 => 1, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 3 => 6, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 5 => 10, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 20 => 30, 10.0 => 20.0, Easing::Linear),
        ];

        assert_eq!(
            check_overlap(&events),
            Err(EventSequenceError::Overlap(beat!(5)))
        );

        let events = vec![
            event!(LineEventKind::X, 0 => 1, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 3 => 6, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 8 => 25, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 20 => 30, 10.0 => 20.0, Easing::Linear),
        ];

        assert_eq!(
            check_overlap(&events),
            Err(EventSequenceError::Overlap(beat!(20)))
        );
    }

    #[test]
    fn test_ensure_same_kind_ok() {
        let events = vec![
            event!(LineEventKind::X, 0 => 1, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 3 => 6, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 8 => 10, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 20 => 30, 10.0 => 20.0, Easing::Linear),
        ];

        assert!(ensure_same_kind(&events).is_ok());
    }

    #[test]
    fn test_ensure_same_kind_err() {
        let events = vec![
            event!(LineEventKind::X, 0 => 1, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 3 => 6, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::Y, 8 => 10, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 20 => 30, 10.0 => 20.0, Easing::Linear),
        ];

        assert_eq!(
            ensure_same_kind(&events),
            Err(EventSequenceError::DifferentKind)
        );
    }

    #[test]
    fn test_fill_gap_until() {
        let default = 0.0;
        let events = vec![
            event!(LineEventKind::X, 1 => 2, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 3 => 6, 30.0 => 40.0, Easing::Linear),
            event!(LineEventKind::X, 8 => 10, 50.0 => 60.0, Easing::Linear),
            event!(LineEventKind::X, 20 => 30, 70.0 => 80.0, Easing::Linear),
        ];

        assert_eq!(
            fill_gap_until(&events, beat!(40), default),
            Ok(vec![
                event!(LineEventKind::X, 0 => 1, default), // filled
                event!(LineEventKind::X, 1 => 2, 10.0 => 20.0, Easing::Linear),
                event!(LineEventKind::X, 2 => 3, 20.0), // filled
                event!(LineEventKind::X, 3 => 6, 30.0 => 40.0, Easing::Linear),
                event!(LineEventKind::X, 6 => 8, 40.0), // filled
                event!(LineEventKind::X, 8 => 10, 50.0 => 60.0, Easing::Linear),
                event!(LineEventKind::X, 10 => 20, 60.0), // filled
                event!(LineEventKind::X, 20 => 30, 70.0 => 80.0, Easing::Linear),
                event!(LineEventKind::X, 30 => 40, 80.0), // filled
            ])
        );
    }

    #[test]
    fn test_fill_gap() {
        let default = 0.0;
        let events = vec![
            event!(LineEventKind::X, 1 => 2, 10.0 => 20.0, Easing::Linear),
            event!(LineEventKind::X, 3 => 6, 30.0 => 40.0, Easing::Linear),
            event!(LineEventKind::X, 8 => 10, 50.0 => 60.0, Easing::Linear),
            event!(LineEventKind::X, 20 => 30, 70.0 => 80.0, Easing::Linear),
        ];

        assert_eq!(
            fill_gap(&events, default),
            Ok(vec![
                event!(LineEventKind::X, 0 => 1, default), // filled
                event!(LineEventKind::X, 1 => 2, 10.0 => 20.0, Easing::Linear),
                event!(LineEventKind::X, 2 => 3, 20.0), // filled
                event!(LineEventKind::X, 3 => 6, 30.0 => 40.0, Easing::Linear),
                event!(LineEventKind::X, 6 => 8, 40.0), // filled
                event!(LineEventKind::X, 8 => 10, 50.0 => 60.0, Easing::Linear),
                event!(LineEventKind::X, 10 => 20, 60.0), // filled
                event!(LineEventKind::X, 20 => 30, 70.0 => 80.0, Easing::Linear),
            ])
        );
    }

    #[test]
    fn test_cut_constant() {
        let result = cut(event!(LineEventKind::X, 6 => 8, 40.0));

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], event!(LineEventKind::X, 6 => 8, 40.0));
    }

    #[test]
    fn test_cut_linear() {
        let result = cut(event!(LineEventKind::X, 20 => 30, 70.0 => 80.0, Easing::Linear));

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            event!(LineEventKind::X, 20 => 30, 70.0 => 80.0, Easing::Linear)
        );
    }

    #[test]
    fn test_cut_same_start_end() {
        let result = cut(event!(LineEventKind::X, 20 => 30, 80.0 => 80.0, Easing::EaseInOutSine));

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], event!(LineEventKind::X, 20 => 30, 80.0));
    }

    #[test]
    fn test_cut_easing() {
        let result = cut(event!(LineEventKind::X, 20 => 30, 80.0 => 90.0, Easing::EaseInOutSine));

        assert_eq!(result.len(), 320);
    }
}
