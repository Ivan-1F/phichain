use bevy::prelude::*;

use crate::project::project_loaded;

use self::create_note::create_note_system;

mod create_note;

pub struct EditingPlugin;

impl Plugin for EditingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, create_note_system.run_if(project_loaded()));
    }
}
