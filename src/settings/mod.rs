use crate::misc::WorkingDirectory;
use bevy::prelude::*;
use bevy_persistent::{Persistent, StorageFormat};
use serde::{Deserialize, Serialize};

pub struct EditorSettingsPlugin;

impl Plugin for EditorSettingsPlugin {
    fn build(&self, app: &mut App) {
        let config_dir = app
            .world
            .resource::<WorkingDirectory>()
            .config()
            .expect("Failed to locate config directory");

        app.insert_resource(
            Persistent::<EditorSettings>::builder()
                .name("Editor Settings")
                .format(StorageFormat::Yaml)
                .path(config_dir.join("settings.yml"))
                .default(EditorSettings::default())
                .build()
                .expect("Failed to initialize editor settings"),
        );
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub timeline_scroll_sensitivity: f32,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            timeline_scroll_sensitivity: 10.0,
        }
    }
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub music_volume: f32,
    pub hit_sound_volume: f32,
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct GraphicsSettings {
    pub fullscreen: bool,
    pub vsync: bool,
}

#[derive(Resource, Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct EditorSettings {
    pub general: GeneralSettings,
    pub audio: AudioSettings,
    pub graphics: GraphicsSettings,
}
