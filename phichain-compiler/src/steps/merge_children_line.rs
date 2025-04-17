use crate::utils::EventSequence;
use nalgebra::{Isometry2, Rotation2, Vector2};
use phichain_chart::beat;
use phichain_chart::easing::Easing;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
use phichain_chart::serialization::{PhichainChart, SerializedLine};

fn merge(parent: SerializedLine) -> Vec<SerializedLine> {
    if parent.children.is_empty() {
        vec![parent]
    } else {
        let children = parent
            .children
            .iter()
            .flat_map(|x| merge(x.clone()))
            .collect::<Vec<_>>();

        let mut merged_children = vec![];

        for child in children {
            let mut merged_move_events = vec![];
            let mut merged_rotate_events = vec![];

            let mut splits = vec![];

            for event in parent
                .events
                .iter()
                .filter(|x| x.kind.is_x() || x.kind.is_y() || x.kind.is_rotation())
            {
                splits.push(event.start_beat);
                splits.push(event.end_beat);
            }
            for event in child
                .events
                .iter()
                .filter(|x| x.kind.is_x() || x.kind.is_y() || x.kind.is_rotation())
            {
                splits.push(event.start_beat);
                splits.push(event.end_beat);
            }

            splits.sort();
            splits.dedup();

            let minimum = beat!(1, 32);

            if let (Some(first), Some(last)) = (splits.first().copied(), splits.last().copied()) {
                let mut current = first;
                while current < last {
                    let start_beat = current;
                    let end_beat = current + minimum;

                    macro_rules! evaluate {
                        ($target:ident, $filter:ident) => {
                            (
                                $target
                                    .events
                                    .$filter()
                                    .evaluate_start_no_effect(start_beat),
                                $target.events.$filter().evaluate(end_beat),
                            )
                        };
                    }

                    macro_rules! evaluate_line {
                        ($target:ident) => {{
                            let (start_x, end_x) = evaluate!($target, x);
                            let (start_y, end_y) = evaluate!($target, y);
                            let (start_rotation, end_rotation) = evaluate!($target, rotation);

                            let start = Isometry2::new(
                                Vector2::new(start_x, start_y),
                                Rotation2::new(start_rotation.to_radians()).angle(),
                            );
                            let end = Isometry2::new(
                                Vector2::new(end_x, end_y),
                                Rotation2::new(end_rotation.to_radians()).angle(),
                            );

                            (start, end)
                        }};
                    }

                    let (parent_start, parent_end) = evaluate_line!(parent);
                    let (child_start, child_end) = evaluate_line!(child);

                    let start = parent_start * child_start;
                    let end = parent_end * child_end;

                    merged_move_events.push(LineEvent {
                        kind: LineEventKind::X,
                        start_beat,
                        end_beat,
                        value: LineEventValue::transition(
                            start.translation.x,
                            end.translation.x,
                            Easing::Linear,
                        ),
                    });
                    merged_move_events.push(LineEvent {
                        kind: LineEventKind::Y,
                        start_beat,
                        end_beat,
                        value: LineEventValue::transition(
                            start.translation.y,
                            end.translation.y,
                            Easing::Linear,
                        ),
                    });
                    merged_rotate_events.push(LineEvent {
                        kind: LineEventKind::Rotation,
                        start_beat,
                        end_beat,
                        value: LineEventValue::transition(
                            start.rotation.angle().to_degrees(),
                            end.rotation.angle().to_degrees(),
                            Easing::Linear,
                        ),
                    });

                    current += minimum;
                }
            }

            let other_events = child
                .events
                .iter()
                .filter(|x| !x.kind.is_x() && !x.kind.is_y() && !x.kind.is_rotation())
                .cloned()
                .collect::<Vec<_>>();

            let merged = SerializedLine {
                events: [other_events, merged_move_events, merged_rotate_events].concat(),
                children: vec![],
                ..child
            };

            merged_children.push(merged);
        }

        merged_children.push(parent);

        merged_children
    }
}

/// Flatten all children lines into the root level, calculate event propagation for X, Y and Rotate events
pub fn merge_children_line(chart: PhichainChart) -> PhichainChart {
    let mut lines = vec![];

    for line in chart.lines {
        lines.append(&mut merge(line));
    }

    PhichainChart { lines, ..chart }
}
