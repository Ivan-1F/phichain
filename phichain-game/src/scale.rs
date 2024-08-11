use crate::{GameConfig, GameSet, GameViewport};
use bevy::prelude::*;

pub struct ScalePlugin;

impl Plugin for ScalePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NoteScale(1.0))
            .add_systems(Update, update_note_scale_system.in_set(GameSet));
    }
}

#[derive(Debug, Resource)]
pub struct NoteScale(pub f32);

fn update_note_scale_system(
    viewport: Res<GameViewport>,
    mut scale: ResMut<NoteScale>,
    config: Res<GameConfig>,
) {
    scale.0 = viewport.0.width() / 8000.0 * config.note_scale
}
