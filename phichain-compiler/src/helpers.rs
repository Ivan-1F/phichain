use phichain_chart::easing::Easing;
use phichain_chart::event::{LineEvent, LineEventValue};

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
                .evaluate(event.start_beat.value())
                .value()
                .unwrap();
            let expected_end = target_event
                .evaluate(event.end_beat.value())
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
