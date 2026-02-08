use crate::compile::steps::evaluate_curve_note_tracks;
use crate::rpe::schema::{
    RpeBpmPoint, RpeChart, RpeCommonEvent, RpeEventLayer, RpeJudgeLine, RpeMeta, RpeNote,
    RpeNoteKind, RpeSpeedEvent, RPE_EASING,
};
use phichain_chart::easing::Easing;
use phichain_chart::event::{LineEvent, LineEventKind};
use phichain_chart::note::{Note, NoteKind};
use phichain_chart::serialization::{PhichainChart, SerializedLine};
use tracing::warn;

struct RpeEasingInfo {
    easing_type: i32,
    bezier: i32,
    bezier_points: [f32; 4],
}

fn easing(easing: Easing) -> RpeEasingInfo {
    match easing {
        // RPE custom bezier: easingType = 1 + bezier = 1 + bezierPoints
        Easing::Custom(a, b, c, d) => RpeEasingInfo {
            easing_type: 1,
            bezier: 1,
            bezier_points: [a, b, c, d],
        },
        _ => {
            let easing_type = RPE_EASING
                .iter()
                .position(|x| x == &easing)
                .unwrap_or_else(|| {
                    warn!("Unsupported easing type: {}", easing);
                    0
                }) as i32;
            RpeEasingInfo {
                easing_type,
                bezier: 0,
                bezier_points: [0.0, 0.0, 0.0, 0.0],
            }
        }
    }
}

fn common_event_from_line_event(event: &LineEvent) -> RpeCommonEvent<f32> {
    let easing_info = easing(event.value.easing());

    RpeCommonEvent {
        bezier: easing_info.bezier,
        bezier_points: easing_info.bezier_points,
        easing_type: easing_info.easing_type,
        start: event.value.start(),
        start_time: event.start_beat.into(),
        end: event.value.end(),
        end_time: event.end_beat.into(),
    }
}

fn note(note: &Note) -> RpeNote {
    RpeNote {
        above: if note.above { 1 } else { 0 },
        start_time: note.beat.into(),
        end_time: match note.kind {
            NoteKind::Hold { hold_beat } => (note.beat + hold_beat).into(),
            _ => note.beat.into(),
        },
        position_x: note.x,
        speed: note.speed,
        // TODO: impl Into<RpeNoteKind> for NoteKind
        kind: match note.kind {
            NoteKind::Tap => RpeNoteKind::Tap,
            NoteKind::Drag => RpeNoteKind::Drag,
            NoteKind::Hold { .. } => RpeNoteKind::Hold,
            NoteKind::Flick => RpeNoteKind::Flick,
        },

        // FIXME: using default Default impl
        ..Default::default()
    }
}

fn event_layer_from_line(line: &SerializedLine) -> RpeEventLayer {
    let mut event_layer = RpeEventLayer::default();

    for event in &line.events {
        match event.kind {
            LineEventKind::Speed => {
                event_layer.speed_events.push(RpeSpeedEvent {
                    start: event.value.start(),
                    start_time: event.start_beat.into(),
                    end: event.value.end(),
                    end_time: event.end_beat.into(),
                });
            }
            _ => {
                let rpe_event = common_event_from_line_event(event);

                match event.kind {
                    LineEventKind::X => {
                        event_layer.move_x_events.push(rpe_event);
                    }
                    LineEventKind::Y => {
                        event_layer.move_y_events.push(rpe_event);
                    }
                    LineEventKind::Rotation => {
                        event_layer.rotate_events.push(rpe_event);
                    }
                    LineEventKind::Opacity => {
                        let easing_info = easing(event.value.easing());
                        event_layer.alpha_events.push(RpeCommonEvent {
                            bezier: easing_info.bezier,
                            bezier_points: easing_info.bezier_points,
                            easing_type: easing_info.easing_type,
                            start: event.value.start() as i32,
                            start_time: event.start_beat.into(),
                            end: event.value.end() as i32,
                            end_time: event.end_beat.into(),
                        });
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    event_layer
}

fn push_line(line: &SerializedLine, parent_index: Option<usize>, target: &mut Vec<RpeJudgeLine>) {
    let event_layer = event_layer_from_line(line);
    let current_index = target.len();
    target.push(RpeJudgeLine {
        name: line.line.name.clone(),
        father: parent_index.map(|i| i as i32).unwrap_or(-1),
        rotate_with_father: true,
        event_layers: vec![event_layer],
        notes: line.notes.iter().map(note).collect(),
        // does not include holds, ref: https://teamflos.github.io/phira-docs/chart-standard/chart-format/rpe/judgeLine.html
        num_of_notes: line
            .notes
            .iter()
            .filter(|note| !note.kind.is_hold())
            .count(),
        attach_ui: None,
    });

    for child in &line.children {
        push_line(child, Some(current_index), target);
    }
}

pub fn phichain_to_rpe(phichain: PhichainChart) -> RpeChart {
    let phichain = evaluate_curve_note_tracks(phichain);

    let mut rpe = RpeChart {
        bpm_list: phichain
            .bpm_list
            .0
            .iter()
            .map(|bpm_point| RpeBpmPoint {
                bpm: bpm_point.bpm,
                start_time: bpm_point.beat.into(),
            })
            .collect(),
        meta: RpeMeta {
            offset: phichain.offset.0 as i32,
            ..RpeMeta::default()
        },
        judge_line_list: vec![],
    };

    for line in &phichain.lines {
        push_line(line, None, &mut rpe.judge_line_list);
    }

    rpe
}
