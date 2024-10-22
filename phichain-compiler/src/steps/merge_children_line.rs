use crate::utils::EventSequence;
use nalgebra::{Isometry, Isometry2, Rotation2, UnitComplex, Vector2};
use phichain_chart::beat;
use phichain_chart::easing::Easing;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
use phichain_chart::serialization::{LineWrapper, PhichainChart};

fn transform(
    parent_position: Vector2<f32>,
    child_position: Vector2<f32>,
    parent_rotation: Rotation2<f32>,
    child_rotation: Rotation2<f32>,
) -> Isometry<f32, UnitComplex<f32>, 2> {
    let parent_isometry = Isometry2::new(parent_position, parent_rotation.angle());
    let child_isometry = Isometry2::new(child_position, child_rotation.angle());

    parent_isometry * child_isometry
}

fn merge(parent: LineWrapper) -> Vec<LineWrapper> {
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

            splits.dedup();
            splits.sort();

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

                    let (parent_x_start, parent_x_end) = evaluate!(parent, x);
                    let (parent_y_start, parent_y_end) = evaluate!(parent, y);
                    let (parent_rotation_start, parent_rotation_end) = evaluate!(parent, rotation);

                    let (child_x_start, child_x_end) = evaluate!(child, x);
                    let (child_y_start, child_y_end) = evaluate!(child, y);
                    let (child_rotation_start, child_rotation_end) = evaluate!(child, rotation);

                    let start = transform(
                        Vector2::new(parent_x_start, parent_y_start),
                        Vector2::new(child_x_start, child_y_start),
                        Rotation2::new(parent_rotation_start.to_radians()),
                        Rotation2::new(child_rotation_start.to_radians()),
                    );
                    let end = transform(
                        Vector2::new(parent_x_end, parent_y_end),
                        Vector2::new(child_x_end, child_y_end),
                        Rotation2::new(parent_rotation_end.to_radians()),
                        Rotation2::new(child_rotation_end.to_radians()),
                    );

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

            let merged = LineWrapper {
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
