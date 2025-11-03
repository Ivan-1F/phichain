use itertools::Itertools;
use phichain_chart::beat;
use phichain_chart::beat::Beat;
use phichain_chart::easing::Easing;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
use thiserror::Error;

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

/// Try to fit a sequence of linear events into a single eased event
///
/// Returns the fitted event if successful, or the original events if fitting fails.
/// The fitting considers:
/// - All events must have the same duration
/// - All events must be increasing or all decreasing
/// - All events must be consecutive (end_beat == next.start_beat)
/// - Values must be continuous (last.end ≈ next.start)
///
/// ```text
/// In:  |----|----|----|----|----| (many small linear segments)
/// Out: |~~~~~~~~~~~~~~~~~~~~~~~~| (single eased event)
/// ```
pub fn fit_easing(events: &[LineEvent], epsilon: f32) -> Result<LineEvent, Vec<LineEvent>> {
    if events.is_empty() {
        return Err(vec![]);
    }

    if events.len() == 1 {
        return Err(events.to_vec());
    }

    let first = events[0];
    let last = events[events.len() - 1];

    for easing in EASING_FITTING_POSSIBLE_EASINGS {
        let target_event = LineEvent {
            kind: first.kind,
            start_beat: first.start_beat,
            end_beat: last.end_beat,
            value: LineEventValue::transition(first.value.start(), last.value.end(), easing),
        };

        let mut fits = true;
        for event in events {
            let expected_start = target_event
                .evaluate_inclusive(event.start_beat.value())
                .value()
                .unwrap();
            let expected_end = target_event
                .evaluate_inclusive(event.end_beat.value())
                .value()
                .unwrap();

            if (expected_start - event.value.start()).abs() > epsilon
                || (expected_end - event.value.end()).abs() > epsilon
            {
                fits = false;
                break;
            }
        }

        if fits {
            return Ok(target_event);
        }
    }

    Err(events.to_vec())
}

/// Remove events that satisfy the predicate
///
/// ```text
/// In:  [A, B, C, D, E]
/// Predicate: is_even
/// Out: [A, C, E]
/// ```
pub fn remove_if<P>(events: &[LineEvent], mut predicate: P) -> Vec<LineEvent>
where
    P: FnMut(&LineEvent) -> bool,
{
    events.iter().copied().filter(|e| !predicate(e)).collect()
}

/// Map events conditionally: if predicate returns true, apply the function; otherwise keep the event unchanged
///
/// ```text
/// In:  [A, B, C, D, E]
/// Predicate: is_even
/// Transform: to_uppercase
/// Out: [A, B_UPPER, C, D_UPPER, E]
/// ```
pub fn map_if<P, F>(events: &[LineEvent], mut predicate: P, mut f: F) -> Vec<LineEvent>
where
    P: FnMut(&LineEvent) -> bool,
    F: FnMut(LineEvent) -> LineEvent,
{
    events
        .iter()
        .copied()
        .map(|e| if predicate(&e) { f(e) } else { e })
        .collect()
}

/// Check if two events are temporally adjacent
///
/// Returns true if `first` ends exactly when `second` starts, or `first` starts exactly when `second` ends
///
/// This only checks temporal adjacency, not value continuity.
pub fn are_adjacent(first: &LineEvent, second: &LineEvent) -> bool {
    first.end_beat == second.start_beat || first.start_beat == second.end_beat
}

/// Check if two events are contiguous (both beats and values align)
///
/// Returns true if the events are both temporally adjacent AND their values connect smoothly:
/// - `first` ends exactly when `second` starts, with `first.end_value == second.start_value`, or
/// - `first` starts exactly when `second` ends, with `first.start_value == second.end_value`
///
/// This is stricter than [`are_adjacent`] as it requires value continuity.
/// ```
pub fn are_contiguous(first: &LineEvent, second: &LineEvent) -> bool {
    (first.end_beat == second.start_beat && first.value.end() == second.value.start())
        || (first.start_beat == second.end_beat && first.value.start() == second.value.end())
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum EventSequenceError {
    #[error("the event sequence has overlap at {0:?}")]
    Overlap(Beat),
    #[error("events in the event sequence do not share a single kind")]
    DifferentKind,
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
#[derive(Debug, Clone, Copy, Default)]
pub struct CutOptions {
    /// Force splitting linear transitions even if they are eased linearly
    pub force_linear: bool,
}

pub fn cut_with_options(event: LineEvent, minimum: Beat, options: CutOptions) -> Vec<LineEvent> {
    match event.value {
        LineEventValue::Constant(_) => vec![event],
        LineEventValue::Transition { start, end, easing } => {
            if matches!(easing, Easing::Linear) && !(options.force_linear && start != end) {
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

            let mut events = vec![];
            let mut current = event.start_beat;

            while current + minimum <= event.end_beat {
                let start_beat = current;
                let end_beat = current + minimum;

                let start_value = event
                    .evaluate_inclusive(start_beat.value())
                    .value()
                    .unwrap();
                let end_value = event.evaluate_inclusive(end_beat.value()).value().unwrap();

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

pub fn cut(event: LineEvent, minimum: Beat) -> Vec<LineEvent> {
    cut_with_options(event, minimum, CutOptions::default())
}

pub fn sorted(events: &[LineEvent]) -> Vec<LineEvent> {
    events
        .iter()
        .sorted_by_key(|x| x.start_beat)
        .copied()
        .collect()
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
pub fn fill_gap(events: &[LineEvent], default: f32) -> Result<Vec<LineEvent>, EventSequenceError> {
    fill_gap_until(
        events,
        events.last().map(|x| x.end_beat).unwrap_or(beat!(0)),
        default,
    )
}
