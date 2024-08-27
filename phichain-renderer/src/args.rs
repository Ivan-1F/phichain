use bevy::prelude::Resource;
use clap::Parser;

/// Render Phigros charts into videos
#[derive(Debug, Clone, Parser, Resource)]
pub struct Args {
    // ------ Video Config ------
    /// The path to the Phichain project
    pub path: String,

    /// The path of the output video
    #[arg(short, long, default_value = "output.mp4")]
    pub output: String,

    /// The width of the video
    #[arg(long, default_value_t = 1920)]
    pub width: u32,
    /// The height of the video
    #[arg(long, default_value_t = 1080)]
    pub height: u32,

    /// The fps of the video
    #[arg(long, default_value_t = 60)]
    pub fps: u32,

    /// The start time of the chart to render in seconds. 0.0 if not given
    #[arg(long)]
    pub from: Option<f32>,
    /// The end time of the chart to render in seconds. the duration of the music if not given
    #[arg(long)]
    pub to: Option<f32>,

    // ------ Game Config ------
    /// The scale factor for notes
    #[arg(long, default_value_t = 1.0)]
    pub note_scale: f32,
    /// Whether to enable the FC/AP indicator. Since phichain-renderer always use autoplay, enabling this will result in a constant yellow line
    #[arg(long, default_value_t = true)]
    pub fc_ap_indicator: bool,
    /// Whether to enable multi highlight for notes
    #[arg(long, default_value_t = true)]
    pub multi_highlight: bool,
    /// Whether to hide hit effects
    #[arg(long, default_value_t = false)]
    pub hide_hit_effect: bool,
    /// Overwrite the name of the chart
    #[arg(long)]
    pub name: Option<String>,
    /// Overwrite the level of the chart
    #[arg(long)]
    pub level: Option<String>,
}
