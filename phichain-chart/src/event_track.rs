//! Every event owns a non-empty range of beats on its track, shared with no other event.
//!
//! A track is the events of one kind on one line. Ordered by start beat, its events satisfy `start < end <= next.start`.

use crate::beat::Beat;
use crate::event::{LineEvent, LineEventValue};
use crate::id::Identified;
use crate::serialization::{PhichainChart, SerializedLine};
use tracing::warn;

impl PhichainChart {
    /// Rewrite every track to satisfy the invariant without changing its values, logging each change
    ///
    /// On each beat, the event starting last at or before it decides the value, the later one in the file on a tie.
    /// These values are kept, except inside a shortened transition with non-linear easing.
    pub fn normalize_events(&mut self) {
        fn visit(lines: &mut [SerializedLine]) {
            for line in lines {
                normalize_line(line);
                visit(&mut line.children);
            }
        }

        visit(&mut self.lines);
    }
}

fn normalize_line(line: &mut SerializedLine) {
    let mut keep = vec![true; line.events.len()];
    for (index, next) in with_next_start(&line.events) {
        let Identified { id, data: event } = &mut line.events[index];
        let has_length = event.end_beat > event.start_beat;
        // the end value holds once an event ends, so any length keeps it
        let mut end = if has_length {
            event.end_beat
        } else {
            event.start_beat + Beat::ONE
        };
        if let Some(next) = next {
            end = end.min(next);
        }

        if end <= event.start_beat {
            warn!(
                "removed event {id} on line {}, which starts together with a later one",
                line.id
            );
            keep[index] = false;
        } else if !has_length {
            warn!(
                "extended event {id} on line {} without length to {end:?}",
                line.id
            );
            event.value = LineEventValue::constant(event.value.end());
            event.end_beat = end;
        } else if end < event.end_beat {
            warn!(
                "shortened event {id} on line {} to {end:?}, where the next one starts",
                line.id
            );
            let value = event.value_at(end.value());
            *event.value.end_mut() = value;
            event.end_beat = end;
        }
    }

    let mut keep = keep.into_iter();
    line.events.retain(|_| keep.next().unwrap());
}

/// Indices of the events ordered by track, start beat and position, each with the start of the next event on its track
fn with_next_start(events: &[Identified<LineEvent>]) -> Vec<(usize, Option<Beat>)> {
    let mut order: Vec<usize> = (0..events.len()).collect();
    order.sort_unstable_by_key(|&index| {
        let event = &events[index].data;
        (u8::from(event.kind), event.start_beat, index)
    });
    order
        .chunk_by(|&a, &b| events[a].data.kind == events[b].data.kind)
        .flat_map(|track| {
            track.iter().enumerate().map(move |(position, &index)| {
                let next = track.get(position + 1);
                (index, next.map(|&next| events[next].data.start_beat))
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::beat;
    use crate::event;
    use crate::event::LineEventKind::{self, X, Y};
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    fn chart(events: Vec<LineEvent>) -> PhichainChart {
        let line = SerializedLine {
            events: events.into_iter().map(Identified::new).collect(),
            ..Default::default()
        };
        PhichainChart::new(0.0, Default::default(), vec![line])
    }

    fn events(line: &SerializedLine) -> Vec<LineEvent> {
        line.events.iter().map(|event| event.data).collect()
    }

    /// Every event has length and shares no beat with another event of its kind
    fn is_valid(line: &SerializedLine) -> bool {
        let events = events(line);
        events.iter().enumerate().all(|(index, a)| {
            a.start_beat < a.end_beat
                && events[index + 1..].iter().all(|b| {
                    a.kind != b.kind || a.end_beat <= b.start_beat || b.end_beat <= a.start_beat
                })
        })
    }

    /// The value decided by the event starting last at or before the beat
    fn value_at(events: &[Identified<LineEvent>], kind: LineEventKind, beat: Beat) -> Option<f32> {
        let (_, event) = events
            .iter()
            .map(|event| event.data)
            .enumerate()
            .filter(|(_, event)| event.kind == kind && event.start_beat <= beat)
            .max_by_key(|&(index, event)| (event.start_beat, index))?;
        Some(event.value_at(beat.value()))
    }

    #[test]
    fn valid_tracks_stay_unchanged() {
        let mut chart = chart(vec![
            event!(X, beat!(0) => beat!(2), 0 => 1),
            event!(X, beat!(2) => beat!(3), 5),
            event!(X, beat!(4) => beat!(6), 2 => 3),
            event!(Y, beat!(1) => beat!(5), 1),
        ]);
        let before = events(&chart.lines[0]);
        chart.normalize_events();
        assert_eq!(events(&chart.lines[0]), before);
    }

    #[test]
    fn child_lines_are_normalized() {
        let mut chart = PhichainChart::default();
        let mut child = SerializedLine::default();
        child
            .events
            .push(Identified::new(event!(X, beat!(0) => beat!(2), 0)));
        chart.lines[0].children.push(child);

        chart.normalize_events();
        assert!(is_valid(&chart.lines[0].children[0]));
    }

    #[test]
    fn each_kind_of_invalid_event_is_rewritten() {
        let mut chart = chart(vec![
            // overlaps the next event
            event!(X, beat!(0) => beat!(4), 0 => 4),
            // starts together with the next event
            event!(X, beat!(2) => beat!(3), 9),
            event!(X, beat!(2) => beat!(5), 1),
            // no length, followed by a gap
            event!(X, beat!(6) => beat!(6), 3 => 7),
            event!(X, beat!(8) => beat!(9), 0),
            // no length, starting together with the next event
            event!(Y, beat!(1) => beat!(1), 5),
            event!(Y, beat!(1) => beat!(2), 0),
            // no length, the last event
            event!(Y, beat!(3) => beat!(3), 2),
        ]);
        chart.normalize_events();
        assert_eq!(
            events(&chart.lines[0]),
            vec![
                event!(X, beat!(0) => beat!(2), 0 => 2),
                event!(X, beat!(2) => beat!(5), 1),
                event!(X, beat!(6) => beat!(7), 7),
                event!(X, beat!(8) => beat!(9), 0),
                event!(Y, beat!(1) => beat!(2), 0),
                event!(Y, beat!(3) => beat!(4), 2),
            ]
        );
    }

    #[test]
    fn normalizing_keeps_every_value() {
        let mut rng = StdRng::seed_from_u64(0);
        for _ in 0..500 {
            // in quarter beats, so ties and seams land on exact beats
            let events: Vec<_> = (0..rng.random_range(0..10))
                .map(|_| {
                    let start = rng.random_range(0..32);
                    let end = start + rng.random_range(0..12);
                    let value = if rng.random_bool(0.5) {
                        LineEventValue::constant(rng.random_range(-10.0..10.0))
                    } else {
                        LineEventValue::transition(
                            rng.random_range(-10.0..10.0),
                            rng.random_range(-10.0..10.0),
                            crate::easing::Easing::Linear,
                        )
                    };
                    LineEvent {
                        kind: if rng.random_bool(0.8) { X } else { Y },
                        start_beat: beat!(0, start, 4),
                        end_beat: beat!(0, end, 4),
                        value,
                    }
                })
                .collect();
            let mut chart = chart(events);
            let before = chart.lines[0].events.clone();
            chart.normalize_events();
            assert!(is_valid(&chart.lines[0]), "{before:?}");

            let after = &chart.lines[0].events;
            for eighth in -2..100 {
                let beat = beat!(0, eighth, 8);
                for kind in [X, Y] {
                    let expected = value_at(&before, kind, beat);
                    let actual = value_at(after, kind, beat);
                    let same = match (expected, actual) {
                        (Some(a), Some(b)) => (a - b).abs() < 1e-4,
                        (a, b) => a == b,
                    };
                    assert!(same, "{expected:?} != {actual:?} at {beat:?} in {before:?}");
                }
            }
        }
    }
}
