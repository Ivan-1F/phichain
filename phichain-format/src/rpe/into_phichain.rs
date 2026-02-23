use crate::rpe::errors::RpeInputError;
use crate::rpe::schema::{
    RpeChart, RpeCommonEvent, RpeEventLayer, RpeNote, RpeNoteKind, RPE_EASING,
};
use crate::rpe::RpeInputOptions;
use num::{Num, ToPrimitive};
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::{BpmList, BpmPoint};
use phichain_chart::easing::Easing;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
use phichain_chart::line::Line;
use phichain_chart::note::{Note, NoteKind};
use phichain_chart::offset::Offset;
use phichain_chart::serialization::{PhichainChart, SerializedLine};
use tracing::warn;

struct LineWithParent {
    line: SerializedLine,
    /// Index of the parent line, -1 means no parent (root line)
    father: i32,
    rotate_with_father: bool,
}

/// Convert RpeNotes to Phichain notes
fn convert_rpe_notes(rpe_notes: &[RpeNote]) -> Result<Vec<Note>, RpeInputError> {
    rpe_notes
        .iter()
        .map(|note| {
            let start_beat: Beat = note.start_time.clone().try_into()?;
            let end_beat: Beat = note.end_time.clone().try_into()?;
            let kind: NoteKind = match note.kind {
                RpeNoteKind::Tap => NoteKind::Tap,
                RpeNoteKind::Drag => NoteKind::Drag,
                RpeNoteKind::Hold => NoteKind::Hold {
                    hold_beat: end_beat - start_beat,
                },
                RpeNoteKind::Flick => NoteKind::Flick,
            };

            Ok(Note::new(
                kind,
                note.above == 1,
                start_beat,
                note.position_x,
                note.speed,
            ))
        })
        .collect()
}

/// Convert a single RpeEventLayer to LineEvents
fn convert_event_layer(
    layer: &RpeEventLayer,
    easing_fn: &impl Fn(i32) -> Easing,
) -> Result<Vec<LineEvent>, RpeInputError> {
    let mut events = Vec::new();

    fn convert_event<T: Num + ToPrimitive>(
        kind: LineEventKind,
        event: RpeCommonEvent<T>,
        easing_fn: &impl Fn(i32) -> Easing,
    ) -> Result<LineEvent, RpeInputError> {
        Ok(LineEvent {
            kind,
            start_beat: event.start_time.try_into()?,
            end_beat: event.end_time.try_into()?,
            value: if event.start == event.end {
                LineEventValue::constant(event.start.to_f32().unwrap_or_default())
            } else {
                LineEventValue::transition(
                    event.start.to_f32().unwrap_or_default(),
                    event.end.to_f32().unwrap_or_default(),
                    easing_fn(event.easing_type),
                )
            },
        })
    }

    // Convert moveX events
    for event in &layer.move_x_events {
        events.push(convert_event(LineEventKind::X, event.clone(), easing_fn)?);
    }

    // Convert moveY events
    for event in &layer.move_y_events {
        events.push(convert_event(LineEventKind::Y, event.clone(), easing_fn)?);
    }

    // Convert rotate events (negate values)
    for event in &layer.rotate_events {
        let mut phichain_event = convert_event(LineEventKind::Rotation, event.clone(), easing_fn)?;

        phichain_event.value = phichain_event.value.negated();

        events.push(phichain_event);
    }

    // Convert alpha events
    for event in &layer.alpha_events {
        events.push(convert_event(
            LineEventKind::Opacity,
            event.clone(),
            easing_fn,
        )?);
    }

    // Convert speed events
    for event in &layer.speed_events {
        events.push(LineEvent {
            kind: LineEventKind::Speed,
            start_beat: event.start_time.clone().try_into()?,
            end_beat: event.end_time.clone().try_into()?,
            value: if event.start == event.end {
                LineEventValue::constant(event.start)
            } else {
                LineEventValue::transition(event.start, event.end, Easing::Linear)
            },
        });
    }

    Ok(events)
}

/// Extract the first event layer, ignoring all other layers
///
/// In RPE, event layers are additive - the final value is the sum of all layers.
/// However, Phichain doesn't support this additive model, so we only take the
/// first layer and discard the rest. This is a lossy conversion.
fn extract_first_layer(event_layers: Vec<RpeEventLayer>) -> RpeEventLayer {
    event_layers.into_iter().next().unwrap_or_default()
}

/// Build a single line from the first event layer only
///
/// This converts only the first RPE event layer into a SerializedLine,
/// discarding all other layers. This is a lossy conversion that ignores
/// the additive semantics of RPE event layers.
fn build_flattened_line(
    line_index: usize,
    line_name: &str,
    event_layers: Vec<RpeEventLayer>,
    notes: Vec<Note>,
    easing_fn: &impl Fn(i32) -> Easing,
) -> Result<SerializedLine, RpeInputError> {
    // Format line name
    let name = if line_name.is_empty() || line_name == "Untitled" {
        format!("#{}", line_index)
    } else {
        format!("{} (#{})", line_name, line_index)
    };

    // Warn if multiple event layers are being discarded
    if event_layers.len() > 1 {
        warn!(
            "Line {} has {} event layers, but only the first layer will be used. \
             Other layers will be discarded.",
            line_index,
            event_layers.len()
        );
    }

    // Take only the first event layer, ignore the rest
    let first_layer = extract_first_layer(event_layers);

    // Convert the first layer to events
    let events = convert_event_layer(&first_layer, easing_fn)?;

    Ok(SerializedLine {
        line: Line { name },
        notes,
        events,
        children: vec![],
        curve_note_tracks: vec![],
    })
}

pub fn rpe_to_phichain(
    rpe: RpeChart,
    options: &RpeInputOptions,
) -> Result<PhichainChart, RpeInputError> {
    let bpm_points: Result<Vec<_>, _> = rpe
        .bpm_list
        .iter()
        .map(|x| -> Result<BpmPoint, RpeInputError> {
            Ok(BpmPoint::new(x.start_time.clone().try_into()?, x.bpm))
        })
        .collect();

    let mut phichain = PhichainChart {
        offset: Offset(rpe.meta.offset as f32),
        bpm_list: BpmList::new(bpm_points?),
        ..PhichainChart::empty()
    };

    let easing_fn = |id: i32| {
        RPE_EASING.get(id as usize).copied().unwrap_or_else(|| {
            warn!("Unknown easing type: {}", id);
            Easing::Linear
        })
    };

    let lines_with_parent: Vec<LineWithParent> = rpe
        .judge_line_list
        .into_iter()
        .enumerate()
        .map(|(index, rpe_line)| -> Result<LineWithParent, RpeInputError> {
            let mut rpe_line = rpe_line.clone();
            // Always warn about UI control lines
            if let Some(attach_ui) = &rpe_line.attach_ui {
                if options.remove_ui_controls {
                    warn!(
                        "Skipping UI control line with attachUI = {:?} (Phichain doesn't support UI control lines)",
                        attach_ui
                    );
                    rpe_line.num_of_notes = 0;
                    rpe_line.notes.clear();
                    rpe_line.event_layers.clear();
                } else {
                    warn!(
                        "Line {} has attachUI = {:?}, but Phichain doesn't support UI control lines. \
                         This line will be treated as a normal line.",
                        index, attach_ui
                    );
                }
            }

            // Filter out fake notes if remove_fake_notes is enabled
            let filtered_notes: Vec<_> = if options.remove_fake_notes {
                let fake_count = rpe_line.notes.iter().filter(|note| note.is_fake == 1).count();
                if fake_count > 0 {
                    warn!(
                        "Line {} has {} fake note(s) that will be removed",
                        index, fake_count
                    );
                }
                rpe_line.notes.iter().filter(|note| note.is_fake != 1).cloned().collect()
            } else {
                rpe_line.notes.clone()
            };

            let notes = convert_rpe_notes(&filtered_notes)?;
            let line = build_flattened_line(index, &rpe_line.name, rpe_line.event_layers, notes, &easing_fn)?;
            Ok(LineWithParent {
                line,
                father: rpe_line.father,
                rotate_with_father: rpe_line.rotate_with_father,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Warn about rotate_with_father = false, as Phichain doesn't currently support this
    for (index, line_with_parent) in lines_with_parent.iter().enumerate() {
        if !line_with_parent.rotate_with_father {
            warn!(
                "Line {} has rotate_with_father = false, but Phichain currently doesn't support \
                 disabling rotation inheritance. The line will inherit its parent's rotation.",
                index
            );
        }
    }

    // Build tree structure
    let lines = build_parent_child_tree(lines_with_parent);

    // Remove lines that came from 0-note + 0-eventLayer inputs (e.g. removed UI control lines),
    // while keeping their children.
    let lines = remove_empty_placeholder_lines(lines);

    phichain.lines = lines;
    Ok(phichain)
}

fn is_empty_placeholder_line(line: &SerializedLine) -> bool {
    line.notes.is_empty() && line.curve_note_tracks.is_empty() && line.events.is_empty()
}

fn remove_empty_placeholder_lines(lines: Vec<SerializedLine>) -> Vec<SerializedLine> {
    fn remove_from_line(mut line: SerializedLine) -> Vec<SerializedLine> {
        line.children = remove_empty_placeholder_lines(line.children);

        if is_empty_placeholder_line(&line) {
            line.children
        } else {
            vec![line]
        }
    }

    lines.into_iter().flat_map(remove_from_line).collect()
}

/// Build a tree structure from flat list of lines with parent indices
fn build_parent_child_tree(lines_with_parent: Vec<LineWithParent>) -> Vec<SerializedLine> {
    if lines_with_parent.is_empty() {
        return vec![];
    }

    // Separate lines and parent information
    let (mut lines, parent_info): (Vec<_>, Vec<_>) = lines_with_parent
        .into_iter()
        .map(|lwp| (Some(lwp.line), lwp.father))
        .unzip();

    // Helper function to build a subtree rooted at a given index
    fn build_subtree(
        index: usize,
        lines: &mut [Option<SerializedLine>],
        parent_info: &[i32],
    ) -> Option<SerializedLine> {
        let mut line = lines[index].take()?;

        // Find all children of this line
        for child_index in 0..parent_info.len() {
            if parent_info[child_index] == index as i32 {
                if let Some(child) = build_subtree(child_index, lines, parent_info) {
                    line.children.push(child);
                }
            }
        }

        Some(line)
    }

    // Find all root lines (father = -1) and build their subtrees
    let mut result = Vec::new();
    for i in 0..parent_info.len() {
        if parent_info[i] == -1 {
            if let Some(root) = build_subtree(i, &mut lines, &parent_info) {
                result.push(root);
            }
        }
    }

    // Promote unmounted lines to roots to avoid silent data loss.
    // This can happen for invalid parent indices, broken parent chains, or cycles
    // that are not reachable from a `father = -1` root.
    for i in 0..parent_info.len() {
        if lines[i].is_some() {
            let father = parent_info[i];
            if father < -1 || father as usize >= parent_info.len() {
                warn!(
                    "Line {} has invalid father index {}. It will be promoted to root.",
                    i, father
                );
            } else {
                warn!(
                    "Line {} is not reachable from any root (father = -1). It will be promoted to root.",
                    i
                );
            }

            if let Some(root) = build_subtree(i, &mut lines, &parent_info) {
                result.push(root);
            }
        }
    }

    result
}
