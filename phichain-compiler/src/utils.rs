use crate::state::LineState;
use phichain_chart::beat::Beat;
use phichain_chart::easing::Easing;
use phichain_chart::event::{LineEvent, LineEventValue};

pub trait EventSequence {
    fn evaluate(&self, beat: Beat) -> f32;
    fn evaluate_state(&self, beat: Beat) -> LineState;
    fn evaluate_start_no_effect(&self, beat: Beat) -> f32;

    fn x(&self) -> Self;
    fn y(&self) -> Self;
    fn rotation(&self) -> Self;
    fn opacity(&self) -> Self;
    fn speed(&self) -> Self;
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

    fn evaluate_state(&self, beat: Beat) -> LineState {
        LineState {
            x: self.x().evaluate(beat),
            y: self.y().evaluate(beat),
            rotation: self.rotation().evaluate(beat),
            opacity: self.opacity().evaluate(beat),
            speed: self.speed().evaluate(beat),
        }
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
}

pub fn event_split_points(events: &Vec<LineEvent>) -> Vec<Beat> {
    let mut splits = vec![];
    for event in events {
        splits.push(event.start_beat);
        splits.push(event.end_beat);
    }

    splits.sort();
    splits.dedup();

    splits
}

pub fn split_event_with_range(
    event: &LineEvent,
    from: Beat,
    to: Beat,
    interval: Beat,
) -> Vec<LineEvent> {
    let mut events = vec![];

    let from = from.clamp(event.start_beat, event.end_beat);
    let to = to.clamp(event.start_beat, event.end_beat);

    let mut current_beat = from;

    while current_beat <= to {
        let start_value = event.evaluate(current_beat.value()).value().unwrap();
        let end_value = event
            .evaluate(current_beat.value() + interval.value())
            .value()
            .unwrap();
        events.push(LineEvent {
            kind: event.kind,
            start_beat: current_beat,
            end_beat: current_beat + interval,
            value: LineEventValue::transition(start_value, end_value, Easing::Linear),
        });
        current_beat += interval;
    }

    events
}

pub fn split_event(event: &LineEvent, interval: Beat) -> Vec<LineEvent> {
    split_event_with_range(event, event.start_beat, event.end_beat, interval)
}
