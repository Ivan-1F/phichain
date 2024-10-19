use crate::utils::EventSequence;
use phichain_chart::beat;
use phichain_chart::easing::Easing;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
use phichain_chart::serialization::{LineWrapper, PhichainChart};

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
                .filter(|x| x.kind.is_x() || x.kind.is_y())
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

                    let parent_x_start = parent
                        .events
                        .iter()
                        .filter(|x| x.kind.is_x())
                        .collect::<Vec<_>>()
                        .evaluate_start_no_effect(start_beat);
                    let parent_y_start = parent
                        .events
                        .iter()
                        .filter(|x| x.kind.is_y())
                        .collect::<Vec<_>>()
                        .evaluate_start_no_effect(start_beat);
                    let parent_x_end = parent
                        .events
                        .iter()
                        .filter(|x| x.kind.is_x())
                        .collect::<Vec<_>>()
                        .evaluate(end_beat);
                    let parent_y_end = parent
                        .events
                        .iter()
                        .filter(|x| x.kind.is_y())
                        .collect::<Vec<_>>()
                        .evaluate(end_beat);

                    let parent_rotation_start = parent
                        .events
                        .iter()
                        .filter(|x| x.kind.is_rotation())
                        .collect::<Vec<_>>()
                        .evaluate_start_no_effect(start_beat)
                        .to_radians();

                    let parent_rotation_end = parent
                        .events
                        .iter()
                        .filter(|x| x.kind.is_rotation())
                        .collect::<Vec<_>>()
                        .evaluate(start_beat)
                        .to_radians();

                    let child_x_start = child
                        .events
                        .iter()
                        .filter(|x| x.kind.is_x())
                        .collect::<Vec<_>>()
                        .evaluate_start_no_effect(start_beat);
                    let child_y_start = child
                        .events
                        .iter()
                        .filter(|x| x.kind.is_y())
                        .collect::<Vec<_>>()
                        .evaluate_start_no_effect(start_beat);
                    let child_x_end = child
                        .events
                        .iter()
                        .filter(|x| x.kind.is_x())
                        .collect::<Vec<_>>()
                        .evaluate(end_beat);
                    let child_y_end = child
                        .events
                        .iter()
                        .filter(|x| x.kind.is_y())
                        .collect::<Vec<_>>()
                        .evaluate(end_beat);

                    // const start = fatherX + childX * Math.cos(fatherR) - childY * Math.sin(fatherR);
                    // const end = fatherX2 + childX2 * Math.cos(fatherR2) - childY2 * Math.sin(fatherR2);
                    // const start2 = fatherY + childX * Math.sin(fatherR) + childY * Math.cos(fatherR);
                    // const end2 = fatherY2 + childX2 * Math.sin(fatherR2) + childY2 * Math.cos(fatherR2);
                    // moveEvents.push({ startTime, endTime, start, end, start2, end2 });

                    let start_x = parent_x_start + child_x_start * parent_rotation_start.cos()
                        - child_y_start * parent_rotation_start.sin();
                    let end_x = parent_x_end + child_x_end * parent_rotation_end.cos()
                        - child_y_end * parent_rotation_end.sin();
                    let start_y = parent_y_start + child_x_start * parent_rotation_start.cos()
                        - child_y_start * parent_rotation_start.sin();
                    let end_y = parent_y_end + child_x_end * parent_rotation_end.cos()
                        - child_y_end * parent_rotation_end.sin();

                    merged_move_events.push(LineEvent {
                        kind: LineEventKind::X,
                        start_beat,
                        end_beat,
                        value: LineEventValue::transition(start_x, end_x, Easing::Linear),
                    });
                    merged_move_events.push(LineEvent {
                        kind: LineEventKind::Y,
                        start_beat,
                        end_beat,
                        value: LineEventValue::transition(start_y, end_y, Easing::Linear),
                    });

                    current += minimum;
                }
            }

            // let events = merged_move_events

            let other_events = child
                .events
                .iter()
                .filter(|x| !x.kind.is_x() && !x.kind.is_y())
                .cloned()
                .collect::<Vec<_>>();

            let merged = LineWrapper {
                events: [other_events, merged_move_events].concat(),
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
