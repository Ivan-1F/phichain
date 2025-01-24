pub mod core;

use self::core::CoreGamePlugin;
use crate::project::project_loaded;
use crate::settings::{AspectRatio, EditorSettings};
use crate::utils;
use crate::utils::convert::BevyEguiConvert;
use bevy::{prelude::*, render::camera::Viewport};
use bevy_persistent::Persistent;
use egui::Ui;

pub fn game_tab(In(ui): In<Ui>, world: &mut World) {
    let aspect_ratio = &world
        .resource::<Persistent<EditorSettings>>()
        .game
        .aspect_ratio;
    let clip_rect = ui.clip_rect();
    let viewport = match aspect_ratio {
        AspectRatio::Free => clip_rect,
        AspectRatio::Fixed { width, height } => {
            utils::misc::keep_aspect_ratio(clip_rect, width / height)
        }
    };

    let mut game_viewport = world.resource_mut::<GameViewport>();
    game_viewport.0 = viewport.into_bevy();

    let mut game_viewport = world.resource_mut::<phichain_game::GameViewport>();
    game_viewport.0 = viewport.into_bevy();
}

pub struct GameTabPlugin;

impl Plugin for GameTabPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameViewport(Rect::from_corners(Vec2::ZERO, Vec2::ZERO)))
            .add_systems(
                PostUpdate,
                update_game_camera_viewport_system.run_if(project_loaded()),
            )
            .add_plugins(CoreGamePlugin);
    }
}

#[derive(Resource, Debug)]
pub struct GameViewport(pub Rect);

#[derive(Component)]
pub struct GameCamera;

pub fn update_game_camera_viewport_system(
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
