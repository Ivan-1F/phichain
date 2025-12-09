use crate::helpers::{are_contiguous, fit_easing};
use crate::sequence::EventSequence;
use phichain_chart::beat::Beat;
use phichain_chart::event::{Direction, LineEvent};

struct Buffer {
    epsilon: f32,
    events: Vec<LineEvent>,
    duration: Option<Beat>,
    direction: Option<Direction>,
}

impl Buffer {
    fn new(epsilon: f32) -> Self {
        Self {
            epsilon,
            events: Vec::new(),
            duration: None,
            direction: None,
        }
    }

    fn accept(&self, event: &LineEvent) -> bool {
        // constant events are not possible to be fitted
        if event.value.is_numeric_constant() {
            return false;
        }

        if let Some(last) = self.events.last() {
            let duration_matches = self
                .duration
                .is_none_or(|duration| duration == event.duration());
            let direction_matches = self
                .direction
                .is_none_or(|direction| direction == event.value.direction());

            are_contiguous(last, event) && duration_matches && direction_matches
        } else {
            // empty buffer
            true
        }
    }

    fn push(&mut self, event: LineEvent) {
        debug_assert!(!event.value.is_numeric_constant());

        if self.events.is_empty() {
            self.duration = Some(event.duration());
            self.direction = Some(event.value.direction());
        }

        self.events.push(event);
    }

    fn drain_into(&mut self, target: &mut Vec<LineEvent>) {
        if self.events.is_empty() {
            return;
        }

        match fit_easing(self.events.as_slice(), self.epsilon) {
            Ok(fitted) => target.push(fitted),
            Err(mut original) => target.append(&mut original),
        }

        self.events.clear();
        self.duration = None;
        self.direction = None;
    }
}

/// Fit a sequence of events by intelligently grouping and fitting them with easings
///
/// Difference from [`fit_easing`]:
/// - [`fit_easing`] operates on a single group of events that are already known to be contiguous
/// - `fit_events` processes an entire event sequence, automatically:
///   - Sorting events by time
///   - Grouping contiguous events with matching duration and direction
///   - Skipping constant events (which cannot be fitted)
///   - Applying [`fit_easing`] to each suitable group
///   - Preserving events that cannot be fitted
pub fn fit_events(events: Vec<LineEvent>, epsilon: f32) -> Vec<LineEvent> {
    if events.is_empty() {
        return vec![];
    }

    let mut fitted_events = Vec::new();
    let mut buffer = Buffer::new(epsilon);

    for event in events.sorted() {
        if !buffer.accept(&event) {
            buffer.drain_into(&mut fitted_events);
        }

        if buffer.accept(&event) {
            buffer.push(event);
        } else {
            fitted_events.push(event);
        }
    }

    buffer.drain_into(&mut fitted_events);

    fitted_events
}
