pub mod core;
pub mod hit_effect;
pub mod illustration;
pub mod scale;
pub mod ui;

use bevy::{prelude::*, render::camera::Viewport};

use crate::project::project_loaded;
use crate::tab::game::hit_effect::HitEffectPlugin;
use crate::tab::game::scale::ScalePlugin;

use self::{core::CoreGamePlugin, illustration::IllustrationPlugin, ui::GameUiPlugin};

pub struct GameTabPlugin;

impl Plugin for GameTabPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameViewport(Rect::from_corners(Vec2::ZERO, Vec2::ZERO)))
            .add_systems(
                PostUpdate,
                update_game_camera_viewport.run_if(project_loaded()),
            )
            .add_plugins(ScalePlugin)
            .add_plugins(GameUiPlugin)
            .add_plugins(IllustrationPlugin)
            .add_plugins(CoreGamePlugin)
            .add_plugins(HitEffectPlugin);
    }
}

#[derive(Resource, Debug)]
pub struct GameViewport(pub Rect);

#[derive(Component)]
pub struct GameCamera;

fn update_game_camera_viewport(
    mut query: Query<&mut Camera, With<GameCamera>>,
    window_query: Query<&Window>,
    egui_settings: Res<bevy_egui::EguiSettings>,
    game_viewport: Res<GameViewport>,
) {
    let mut game_camera = query.single_mut();
    let Ok(window) = window_query.get_single() else {
        return;
    };

    let scale_factor = window.scale_factor() * egui_settings.scale_factor;
    let viewport_pos = game_viewport.0.min * scale_factor;
    let viewport_size = game_viewport.0.size() * scale_factor;

    if viewport_pos.x < 0.0
        || viewport_pos.y < 0.0
        || viewport_size.x <= 0.0
        || viewport_size.y <= 0.0
        || viewport_pos.x + viewport_size.x > window.width() * scale_factor
        || viewport_pos.y + viewport_size.y > window.height() * scale_factor
    {
        return;
    }

    game_camera.viewport = Some(Viewport {
        physical_position: viewport_pos.as_uvec2(),
        physical_size: viewport_size.as_uvec2(),
        depth: 0.0..1.0,
    });
}
