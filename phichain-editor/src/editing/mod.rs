use crate::action::ActionRegistrationExt;
use crate::editing::clipboard::ClipboardPlugin;
use crate::editing::command::EditorCommand;
use crate::editing::create_note::CreateNotePlugin;
use crate::editing::delete_selected::DeleteSelectedPlugin;
use crate::editing::history::{EditorHistory, HistoryPlugin};
use crate::hotkey::modifier::Modifier;
use crate::hotkey::Hotkey;
use crate::schedule::EditorSet;
use bevy::prelude::*;

mod clipboard;
pub mod command;
mod create_event;
mod create_note;
pub mod curve_note_track;
mod delete_selected;
pub mod history;
mod line;
mod moving;
pub mod pending;

pub struct EditingPlugin;

impl Plugin for EditingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DoCommand>()
            .add_plugins(HistoryPlugin)
            .add_plugins(DeleteSelectedPlugin)
            .add_plugins(CreateNotePlugin)
            .add_plugins(ClipboardPlugin)
            .add_systems(Update, reject_legacy_edits.in_set(EditorSet::Edit))
            .add_action(
                "phichain.undo",
                undo_system,
                Some(Hotkey::new(KeyCode::KeyZ, vec![Modifier::Control])),
            )
            .add_action(
                "phichain.redo",
                redo_system,
                Some(Hotkey::new(
                    KeyCode::KeyZ,
                    vec![Modifier::Control, Modifier::Shift],
                )),
            );
    }
}

fn undo_system(world: &mut World) -> Result {
    world.resource_scope(|world, mut history: Mut<EditorHistory>| history.undo(world))?;

    Ok(())
}

fn redo_system(world: &mut World) -> Result {
    world.resource_scope(|world, mut history: Mut<EditorHistory>| history.redo(world))?;

    Ok(())
}

#[derive(Message, Clone)]
pub struct DoCommand(pub EditorCommand);

// Document writes must pass through Edits to be recorded.
fn reject_legacy_edits(mut events: MessageReader<DoCommand>) {
    for event in events.read() {
        warn!("Rejected command outside Edits: {:?}", event.0);
    }
}
