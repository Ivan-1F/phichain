use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;

#[derive(Resource, Default)]
pub struct FpsDisplay {
    pub displayed_fps: f64,
}

pub struct FpsPlugin;

impl Plugin for FpsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FpsDisplay::default()).add_systems(
            Update,
            update_fps_display_system.run_if(on_timer(Duration::from_millis(500))),
        );
    }
}

fn update_fps_display_system(
    diagnostics: Res<DiagnosticsStore>,
    mut fps_display: ResMut<FpsDisplay>,
) {
    if let Some(value) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.average())
    {
        fps_display.displayed_fps = value;
    }
}
