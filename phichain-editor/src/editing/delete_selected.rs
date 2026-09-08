use crate::action::ActionRegistrationExt;
use crate::editing::history::Edits;
use crate::hotkey::Hotkey;
use crate::selection::Selected;
use bevy::prelude::*;
use phichain_game::Pending;

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
    selected: Query<(Entity, Has<Pending>), With<Selected>>,
    mut commands: Commands,
    mut edits: Edits,
) -> Result {
    let mut targets = Vec::new();
    for (entity, pending) in &selected {
        if pending {
            commands.entity(entity).try_despawn();
        } else {
            targets.push(entity);
        }
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
