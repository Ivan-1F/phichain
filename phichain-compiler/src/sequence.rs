use itertools::Itertools;
use phichain_chart::beat::Beat;
use phichain_chart::easing::Easing;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
use std::collections::HashMap;

pub trait EventSequence: Sized {
    fn evaluate(&self, beat: Beat) -> f32;
    fn evaluate_start_no_effect(&self, beat: Beat) -> f32;

    fn x(&self) -> Self;
    fn y(&self) -> Self;
    fn rotation(&self) -> Self;
    fn opacity(&self) -> Self;
    fn speed(&self) -> Self;

    fn group_by_kind(&self) -> HashMap<LineEventKind, Self>;

    fn sorted(&self) -> Self;
}

impl EventSequence for Vec<LineEvent> {
    fn evaluate(&self, beat: Beat) -> f32 {
        let mut ret = 0.0;

        for event in self {
            let result = event.evaluate(beat.value());
            if let Some(value) = result.value() {
                ret = value;
            }
        }

        ret
    }

    fn evaluate_start_no_effect(&self, beat: Beat) -> f32 {
        let mut ret = 0.0;

        for event in self {
            let result = event.evaluate_start_no_effect(beat.value());
            if let Some(value) = result.value() {
                ret = value;
            }
        }

        ret
    }

    fn x(&self) -> Self {
        self.iter().filter(|x| x.kind.is_x()).copied().collect()
    }

    fn y(&self) -> Self {
        self.iter().filter(|x| x.kind.is_y()).copied().collect()
    }

    fn rotation(&self) -> Self {
        self.iter()
            .filter(|x| x.kind.is_rotation())
            .copied()
            .collect()
    }

    fn opacity(&self) -> Self {
        self.iter()
            .filter(|x| x.kind.is_opacity())
            .copied()
            .collect()
    }

    fn speed(&self) -> Self {
        self.iter().filter(|x| x.kind.is_speed()).copied().collect()
    }

    fn group_by_kind(&self) -> HashMap<LineEventKind, Self> {
        let mut map = HashMap::new();

        for event in self {
            map.entry(event.kind).or_insert_with(Vec::new).push(*event);
        }
        map
    }

    fn sorted(&self) -> Self {
        self.iter()
            .sorted_by_key(|x| x.start_beat)
            .copied()
            .collect()
    }
}

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
pub fn fit_easing(
    events: &[LineEvent],
    possible_easings: &[Easing],
    epsilon: f32,
) -> Result<LineEvent, Vec<LineEvent>> {
    if events.is_empty() {
        return Err(vec![]);
    }

    if events.len() == 1 {
        return Err(events.to_vec());
    }

    let first = events[0];
    let last = events[events.len() - 1];

    for &easing in possible_easings {
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
