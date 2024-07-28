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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralSettings {
    pub language: String,
    pub timeline_scroll_sensitivity: f32,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            language: "en_us".to_owned(),
            timeline_scroll_sensitivity: 10.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioSettings {
    pub music_volume: f32,
    pub hit_sound_volume: f32,

    pub playback_rate: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            music_volume: 1.0,
            hit_sound_volume: 1.0,
            playback_rate: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GraphicsSettings {
    pub fullscreen: bool,
    pub vsync: bool,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            vsync: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GameSettings {
    pub fc_ap_indicator: bool,
    pub hide_hit_effect: bool,
    pub multi_highlight: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            fc_ap_indicator: true,
            hide_hit_effect: false,
            multi_highlight: true,
        }
    }
}

#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorSettings {
    pub general: GeneralSettings,
    pub audio: AudioSettings,
    pub graphics: GraphicsSettings,
    pub game: GameSettings,
}
