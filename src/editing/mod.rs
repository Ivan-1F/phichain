use crate::action::ActionRegistrationExt;
use crate::editing::command::EditorCommand;
use crate::editing::delete_selected::DeleteSelectedPlugin;
use crate::editing::history::EditorHistory;
use crate::hotkey::HotkeyRegistrationExt;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;

use crate::project::project_loaded;

use self::create_note::create_note_system;

pub mod command;
mod create_note;
mod delete_selected;
pub mod history;

pub struct EditingPlugin;

impl Plugin for EditingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DoCommandEvent>()
            .init_resource::<EditorHistory>()
            .add_plugins(DeleteSelectedPlugin)
            .add_systems(Update, create_note_system.run_if(project_loaded()))
            .add_systems(Update, handle_edit_command.run_if(project_loaded()))
            .register_action("phichain.undo", undo_system)
            .register_hotkey("phichain.undo", vec![KeyCode::ControlLeft, KeyCode::KeyZ])
            .register_action("phichain.redo", redo_system)
            .register_hotkey(
                "phichain.redo",
                vec![KeyCode::ControlLeft, KeyCode::ShiftLeft, KeyCode::KeyZ],
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
