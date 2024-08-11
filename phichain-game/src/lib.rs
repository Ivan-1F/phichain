use bevy::prelude::*;

#[derive(Debug, Clone, Resource)]
pub struct GameViewport(pub Rect);

#[derive(Debug, Clone, Resource)]
pub struct ChartTime(pub f32);

#[allow(dead_code)]
#[derive(Debug, Clone, Resource)]
pub struct GameConfig {
    note_scale: f32,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameSet;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            GameSet.run_if(
                resource_exists::<GameConfig>
                    .and_then(resource_exists::<ChartTime>)
                    .and_then(resource_exists::<GameViewport>),
            ),
        );
    }
}
