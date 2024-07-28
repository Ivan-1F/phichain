use crate::project::project_loaded;
use crate::settings::EditorSettings;
use crate::tab::game::GameViewport;
use bevy::prelude::*;
use bevy_persistent::Persistent;

pub struct ScalePlugin;

impl Plugin for ScalePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NoteScale(1.0))
            .add_systems(Update, update_note_scale_system.run_if(project_loaded()));
    }
}

#[derive(Debug, Resource)]
pub struct NoteScale(pub f32);

fn update_note_scale_system(
    viewport: Res<GameViewport>,
    mut scale: ResMut<NoteScale>,
    settings: Res<Persistent<EditorSettings>>,
) {
    scale.0 = viewport.0.width() / 8000.0 * settings.game.note_scale
}
