use crate::action::ActionRegistrationExt;
use crate::editing::clipboard::ClipboardPlugin;
use crate::editing::command::EditorCommand;
use crate::editing::create_event::CreateEventPlugin;
use crate::editing::create_note::CreateNoteSystem;
use crate::editing::delete_selected::DeleteSelectedPlugin;
use crate::editing::fill_notes::FillingNotesPlugin;
use crate::editing::history::EditorHistory;
use crate::editing::move_event::MoveEventPlugin;
use crate::editing::move_note::MoveNotePlugin;
use crate::hotkey::HotkeyRegistrationExt;
use crate::schedule::EditorSet;
use crate::utils::compat::ControlKeyExt;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;

mod clipboard;
pub mod command;
mod create_event;
mod create_note;
mod delete_selected;
pub mod fill_notes;
pub mod history;
mod move_event;
mod move_note;
pub mod pending;

pub struct EditingPlugin;

impl Plugin for EditingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DoCommandEvent>()
            .init_resource::<EditorHistory>()
            .add_plugins(DeleteSelectedPlugin)
            .add_plugins(CreateNoteSystem)
            .add_plugins(CreateEventPlugin)
            .add_plugins(MoveNotePlugin)
            .add_plugins(MoveEventPlugin)
            .add_plugins(FillingNotesPlugin)
            .add_plugins(ClipboardPlugin)
            .add_systems(Update, handle_edit_command.in_set(EditorSet::Edit))
            .register_action("phichain.undo", undo_system)
            .register_hotkey("phichain.undo", vec![KeyCode::control(), KeyCode::KeyZ])
            .register_action("phichain.redo", redo_system)
            .register_hotkey(
                "phichain.redo",
                vec![KeyCode::control(), KeyCode::ShiftLeft, KeyCode::KeyZ],
            );
    }
}

fn undo_system(world: &mut World) {
    world.resource_scope(|world, mut history: Mut<EditorHistory>| {
        history.undo(world);
    });
}

fn redo_system(world: &mut World) {
    world.resource_scope(|world, mut history: Mut<EditorHistory>| {
        history.redo(world);
    });
}

#[derive(Event, Clone)]
pub struct DoCommandEvent(pub EditorCommand);

fn handle_edit_command(world: &mut World, state: &mut SystemState<EventReader<DoCommandEvent>>) {
    let events: Vec<_> = {
        let mut event_reader = state.get_mut(world);
        event_reader.read().cloned().collect()
    };

    world.resource_scope(|world, mut history: Mut<EditorHistory>| {
        for event in events {
            history.edit(world, event.0);
        }
    });
}
