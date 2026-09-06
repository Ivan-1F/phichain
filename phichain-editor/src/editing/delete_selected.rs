use crate::action::ActionRegistrationExt;
use crate::editing::history::Edits;
use crate::editing::pending::Pending;
use crate::hotkey::Hotkey;
use crate::notification::{ToastsExt, ToastsStorage};
use crate::selection::Selected;
use bevy::prelude::*;
use phichain_chart::note::Note;
use phichain_game::curve_note_track::{CurveNoteTracksFrom, CurveNoteTracksTo};
use phichain_game::Derived;

pub struct DeleteSelectedPlugin;

impl Plugin for DeleteSelectedPlugin {
    fn build(&self, app: &mut App) {
        app.add_action(
            "phichain.delete_selected",
            delete_selected_system,
            Some(Hotkey::new(KeyCode::Backspace, vec![])),
        );
    }
}

pub(super) fn delete_selected_system(
    selected: Query<
        (
            Entity,
            Option<&Note>,
            Has<Derived>,
            Has<Pending>,
            Option<&CurveNoteTracksFrom>,
            Option<&CurveNoteTracksTo>,
        ),
        With<Selected>,
    >,
    children: Query<&Children>,
    derived_entities: Query<(), With<Derived>>,
    mut edits: Edits,
    mut toasts: ResMut<ToastsStorage>,
) -> Result {
    let mut targets = Vec::new();
    for (entity, note, derived, pending, from, to) in &selected {
        // Validate the entire selection before queueing anything. In particular,
        // deleting an endpoint must not silently lose its unrecorded curve track.
        if note.is_none()
            || derived
            || pending
            || from.is_some_and(|tracks| tracks.iter().next().is_some())
            || to.is_some_and(|tracks| tracks.iter().next().is_some())
            // Render descendants (e.g. a Hold's head and tail) are regenerated
            // after undo. Authored descendants must not be deleted implicitly.
            || children
                .iter_descendants::<Children>(entity)
                .any(|child| !derived_entities.contains(child))
        {
            toasts.info(t!("history.unsupported_delete"));
            return Ok(());
        }
        targets.push(entity);
    }
    if !targets.is_empty() {
        edits.once(
            t!("history.delete_notes", count = targets.len()),
            move |commands| {
                for entity in targets {
                    commands.entity(entity).try_despawn();
                }
            },
        );
    }
    Ok(())
}
