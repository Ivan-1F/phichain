use crate::action::ActionRegistrationExt;
use crate::editing::history::Edits;
use crate::hotkey::Hotkey;
use crate::notification::{ToastsExt, ToastsStorage};
use crate::selection::Selected;
use bevy::prelude::*;
use phichain_chart::event::LineEvent;
use phichain_chart::note::Note;
use phichain_game::curve_note_track::CurveNoteTrackTo;
use phichain_game::{Derived, Pending};

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
    selected: Query<EntityRef, With<Selected>>,
    mut edits: Edits,
    mut toasts: ResMut<ToastsStorage>,
) -> Result {
    let mut targets = Vec::new();
    for entity in &selected {
        if entity.contains::<Derived>()
            || entity.contains::<Pending>()
            || !(entity.contains::<Note>()
                || entity.contains::<LineEvent>()
                || entity.contains::<CurveNoteTrackTo>())
        {
            toasts.info(t!("history.unsupported_delete"));
            return Ok(());
        }
        targets.push(entity.id());
    }
    if !targets.is_empty() {
        edits.once(
            t!("history.delete_objects", count = targets.len()),
            move |commands| {
                for entity in targets {
                    commands.entity(entity).try_despawn();
                }
            },
        );
    }
    Ok(())
}
