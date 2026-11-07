use crate::editing::history::Edits;
use crate::selection::Selected;
use bevy::prelude::*;
use egui::{Align, Layout, Ui};
use phichain_chart::beat;
use phichain_chart::note::{Note, NoteKind};
use phichain_game::{Derived, Pending};

type SelectedNotes<'w, 's> =
    Query<'w, 's, (&'static Note, Entity), (With<Selected>, Without<Derived>, Without<Pending>)>;

fn edit_notes(
    query: &SelectedNotes,
    edits: &mut Edits,
    description: String,
    transform: impl Fn(Note) -> Note,
) {
    let notes: Vec<_> = query
        .iter()
        .map(|(note, entity)| (entity, transform(*note)))
        .collect();
    edits.once(description, move |commands| {
        for (entity, note) in notes {
            commands.entity(entity).insert(note);
        }
    });
}

pub fn multiple_notes_inspector(
    In(mut ui): In<Ui>,
    query: SelectedNotes,
    mut edits: Edits,
) -> Result {
    let count = query.iter().len();
    ui.label(t!("tab.inspector.multiple_notes.title", amount = count));
    ui.separator();
    ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
        if ui
            .button(t!("tab.inspector.multiple_notes.flip_by_x"))
            .clicked()
        {
            edit_notes(
                &query,
                &mut edits,
                t!("history.mirror_notes", count = count).into(),
                |note| Note { x: -note.x, ..note },
            );
        }
        if ui
            .button(t!("tab.inspector.multiple_notes.flip_by_selection"))
            .clicked()
            && count > 0
        {
            let center = query.iter().map(|(note, _)| note.x).sum::<f32>() / count as f32;
            edit_notes(
                &query,
                &mut edits,
                t!("history.mirror_notes", count = count).into(),
                |note| Note {
                    x: 2.0 * center - note.x,
                    ..note
                },
            );
        }
        if ui
            .button(t!("tab.inspector.multiple_notes.flip_side"))
            .clicked()
        {
            edit_notes(
                &query,
                &mut edits,
                t!("history.edit_notes", count = count).into(),
                |note| Note {
                    above: !note.above,
                    ..note
                },
            );
        }
        for (label, kind) in [
            (t!("tab.inspector.multiple_notes.into_tap"), NoteKind::Tap),
            (t!("tab.inspector.multiple_notes.into_drag"), NoteKind::Drag),
            (
                t!("tab.inspector.multiple_notes.into_flick"),
                NoteKind::Flick,
            ),
            (
                t!("tab.inspector.multiple_notes.into_hold"),
                NoteKind::Hold {
                    hold_beat: beat!(1, 32),
                },
            ),
        ] {
            if ui.button(label).clicked() {
                edit_notes(
                    &query,
                    &mut edits,
                    t!("history.edit_notes", count = count).into(),
                    |note| Note { kind, ..note },
                );
            }
        }
    });
    Ok(())
}
