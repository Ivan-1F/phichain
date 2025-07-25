use crate::action::ActionRegistrationExt;
use crate::editing::clipboard::ClipboardPlugin;
use crate::editing::command::EditorCommand;
use crate::editing::create_event::CreateEventPlugin;
use crate::editing::create_note::CreateNotePlugin;
use crate::editing::curve_note_track::CurveNoteTrackPlugin;
use crate::editing::delete_selected::DeleteSelectedPlugin;
use crate::editing::history::EditorHistory;
use crate::editing::line::LineEditingPlugin;
use crate::editing::moving::MovingPlugin;
use crate::hotkey::modifier::Modifier;
use crate::hotkey::Hotkey;
use crate::schedule::EditorSet;
use bevy::ecs::system::SystemState;
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
        app.add_event::<DoCommandEvent>()
            .init_resource::<EditorHistory>()
            .add_plugins(DeleteSelectedPlugin)
            .add_plugins(CreateNotePlugin)
            .add_plugins(CreateEventPlugin)
            .add_plugins(MovingPlugin)
            .add_plugins(CurveNoteTrackPlugin)
            .add_plugins(ClipboardPlugin)
            .add_plugins(LineEditingPlugin)
            .add_systems(Update, handle_edit_command_system.in_set(EditorSet::Edit))
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
    world.resource_scope(|world, mut history: Mut<EditorHistory>| {
        history.undo(world);
    });

    Ok(())
}

fn redo_system(world: &mut World) -> Result {
    world.resource_scope(|world, mut history: Mut<EditorHistory>| {
        history.redo(world);
    });

    Ok(())
}

#[derive(Event, Clone)]
pub struct DoCommandEvent(pub EditorCommand);

fn handle_edit_command_system(
    world: &mut World,
    state: &mut SystemState<EventReader<DoCommandEvent>>,
) {
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
