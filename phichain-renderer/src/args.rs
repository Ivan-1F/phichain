use bevy::prelude::Resource;
use clap::Parser;
use phichain_game::GameConfig;

/// Render Phigros charts into videos
#[derive(Debug, Clone, Parser, Resource)]
pub struct Args {
    // ------ Video Config ------
    /// The path to the Phichain project
    pub path: String,

    /// The path of the output video
    #[arg(short, long, default_value = "output.mp4")]
    pub output: String,

    /// The start time of the chart to render in seconds. 0.0 if not given
    #[arg(long)]
    pub from: Option<f32>,
    /// The end time of the chart to render in seconds. the duration of the music if not given
    #[arg(long)]
    pub to: Option<f32>,

    #[command(flatten)]
    pub video: VideoArgs,

    #[command(flatten)]
    pub game: GameArgs,
}

#[derive(Debug, Clone, Parser)]
#[command(next_help_heading = "Video Options")]
pub struct VideoArgs {
    /// The width of the video
    #[arg(long, default_value_t = 1920)]
    pub width: u32,
    /// The height of the video
    #[arg(long, default_value_t = 1080)]
    pub height: u32,

    /// The fps of the video
    #[arg(long, default_value_t = 60)]
    pub fps: u32,
}

#[derive(Debug, Clone, Parser)]
#[command(next_help_heading = "Game Options")]
pub struct GameArgs {
    /// The scale factor for notes
    #[arg(long, default_value_t = 1.0)]
    pub note_scale: f32,
    /// Enable the FC/AP indicator. Since phichain-renderer always use autoplay, enabling this will result in a constant yellow line
    #[arg(long)]
    pub fc_ap_indicator: bool,
    /// Disable multi highlight for notes
    #[arg(long)]
    pub no_multi_highlight: bool,
    /// Hide hit effects
    #[arg(long)]
    pub hide_hit_effect: bool,
    /// Overwrite the name of the chart
    #[arg(long)]
    pub name: Option<String>,
    /// Overwrite the level of the chart
    #[arg(long)]
    pub level: Option<String>,
}

impl GameArgs {
    pub fn into_game_config(self, name: String, level: String) -> GameConfig {
        GameConfig {
            note_scale: self.note_scale,
            fc_ap_indicator: self.fc_ap_indicator,
            multi_highlight: !self.no_multi_highlight,
            hide_hit_effect: self.hide_hit_effect,
            name: self.name.unwrap_or(name),
            level: self.level.unwrap_or(level),

            hit_effect_follow_game_time: true,
        }
    }
}
