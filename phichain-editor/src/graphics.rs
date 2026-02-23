use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_persistent::Persistent;

use crate::settings::EditorSettings;

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, apply_initial_graphics_settings_system)
            .add_systems(Update, apply_graphics_settings_changes_system);
    }
}

fn apply_initial_graphics_settings_system(
    settings: Res<Persistent<EditorSettings>>,
    mut windows: Query<&mut Window>,
) {
    if let Ok(mut window) = windows.single_mut() {
        window.present_mode = if settings.graphics.vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        };
    }
}

fn apply_graphics_settings_changes_system(
    settings: Res<Persistent<EditorSettings>>,
    mut windows: Query<&mut Window>,
) {
    if settings.is_changed() {
        if let Ok(mut window) = windows.single_mut() {
            // Update UI scale
            let current_resolution = &mut window.resolution;
            current_resolution.set_scale_factor_override(Some(settings.graphics.ui_scale));

            // Force window to apply the new scale by triggering a minimal resize using logical dimensions
            let current_width = current_resolution.width();
            let current_height = current_resolution.height();
            current_resolution.set(current_width, current_height);

            // Update VSync setting
            window.present_mode = if settings.graphics.vsync {
                PresentMode::AutoVsync
            } else {
                PresentMode::AutoNoVsync
            };
        }
    }
}
