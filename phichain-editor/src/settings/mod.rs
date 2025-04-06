use crate::misc::WorkingDirectory;
use bevy::prelude::*;
use bevy_persistent::{Persistent, StorageFormat};
use serde::{Deserialize, Serialize};
use std::fmt;

pub struct EditorSettingsPlugin;

impl Plugin for EditorSettingsPlugin {
    fn build(&self, app: &mut App) {
        let config_dir = app
            .world()
            .resource::<WorkingDirectory>()
            .config()
            .expect("Failed to locate config directory");

        app.insert_resource(
            Persistent::<EditorSettings>::builder()
                .name("Editor Settings")
                .format(StorageFormat::Yaml)
                .path(config_dir.join("settings.yml"))
                .default(EditorSettings::default())
                .revertible(true)
                .revert_to_default_on_deserialization_errors(true) // TODO: better error handling, fix instead of revert
                .build()
                .expect("Failed to initialize editor settings"),
        );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShowLineAnchorOption {
    Never,
    Always,
    Visible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralSettings {
    pub language: String,
    pub timeline_scroll_sensitivity: f32,
    pub highlight_selected_line: bool,
    pub show_line_anchor: ShowLineAnchorOption,

    pub send_telemetry: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            language: "en_us".to_owned(),
            timeline_scroll_sensitivity: 10.0,
            highlight_selected_line: true,
            show_line_anchor: ShowLineAnchorOption::Always,

            send_telemetry: true,
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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AspectRatio {
    #[default]
    Free,
    Fixed {
        width: f32,
        height: f32,
    },
}

impl fmt::Display for AspectRatio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AspectRatio::Free => f.write_str(&t!("game.aspect_ratio.free")),
            AspectRatio::Fixed { width, height } => {
                write!(f, "{}:{}", width, height)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GameSettings {
    pub fc_ap_indicator: bool,
    pub hide_hit_effect: bool,
    pub note_scale: f32,
    pub multi_highlight: bool,
    pub aspect_ratio: AspectRatio,

    pub hit_effect_follow_game_time: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            fc_ap_indicator: true,
            hide_hit_effect: false,
            note_scale: 1.0,
            multi_highlight: true,
            aspect_ratio: AspectRatio::default(),

            hit_effect_follow_game_time: false,
        }
    }
}

#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorSettings {
    pub general: GeneralSettings,
    pub audio: AudioSettings,
    pub game: GameSettings,
}
