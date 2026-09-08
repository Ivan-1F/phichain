use crate::action::ActionRegistrationExt;
use crate::editing::clipboard::ClipboardPlugin;
use crate::editing::create_note::CreateNotePlugin;
use crate::editing::delete_selected::DeleteSelectedPlugin;
use crate::editing::history::{EditorHistory, HistoryPlugin};
use crate::hotkey::modifier::Modifier;
use crate::hotkey::Hotkey;
use bevy::prelude::*;

pub(crate) mod bpm;
mod clipboard;
mod create_event;
mod create_note;
pub mod curve_note_track;
mod delete_selected;
pub mod history;
pub(crate) mod line;
mod moving;
pub(crate) mod project_settings;

pub struct EditingPlugin;

impl Plugin for EditingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HistoryPlugin)
            .add_plugins(DeleteSelectedPlugin)
            .add_plugins(CreateNotePlugin)
            .add_plugins(ClipboardPlugin)
            .add_plugins(create_event::CreateEventPlugin)
            .add_plugins(moving::MovingPlugin)
            .add_plugins(line::LineEditingPlugin)
            .add_plugins(curve_note_track::CurveNoteTrackPlugin)
            .add_plugins(bpm::BpmEditingPlugin)
            .add_plugins(project_settings::ProjectSettingsPlugin)
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
