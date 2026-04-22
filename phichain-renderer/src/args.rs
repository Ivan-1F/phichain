use crate::i18n::i18n_str;
use bevy::prelude::Resource;
use bevy::render::view::Msaa;
use clap::{Parser, ValueEnum};
use phichain_game::GameConfig;
use rust_i18n::t;
use std::path::PathBuf;

#[derive(Debug, Clone, Parser, Resource)]
#[command(about = i18n_str("cli.about"))]
pub struct Args {
    #[arg(help = t!("cli.args.path").to_string())]
    pub path: String,

    #[arg(short, long, default_value = "output.mp4", help = t!("cli.args.output").to_string())]
    pub output: String,

    #[arg(long, help = t!("cli.args.from").to_string())]
    pub from: Option<f32>,
    #[arg(long, help = t!("cli.args.to").to_string())]
    pub to: Option<f32>,

    #[arg(long, help = t!("cli.args.respack").to_string())]
    pub respack: Option<PathBuf>,

    #[command(flatten)]
    #[command(next_help_heading = i18n_str("cli.video.heading"))]
    pub video: VideoArgs,

    #[command(flatten)]
    #[command(next_help_heading = i18n_str("cli.game.heading"))]
    pub game: GameArgs,
}

#[derive(Debug, Clone, Parser)]
pub struct VideoArgs {
    #[arg(
        long,
        default_value_t = 1920,
        value_parser = clap::value_parser!(u32).range(1..=16384),
        help = t!("cli.video.width").to_string(),
    )]
    pub width: u32,
    #[arg(
        long,
        default_value_t = 1080,
        value_parser = clap::value_parser!(u32).range(1..=16384),
        help = t!("cli.video.height").to_string(),
    )]
    pub height: u32,

    #[arg(
        long,
        default_value_t = 60,
        value_parser = clap::value_parser!(u32).range(1..=240),
        help = t!("cli.video.fps").to_string(),
    )]
    pub fps: u32,

    #[arg(long, value_enum, default_value_t = MsaaLevel::Four, help = t!("cli.video.msaa").to_string())]
    pub msaa: MsaaLevel,

    #[arg(long, help = t!("cli.video.hwaccel").to_string())]
    pub hwaccel: bool,

    #[arg(long, value_enum, default_value_t = Codec::H264, help = t!("cli.video.codec").to_string())]
    pub codec: Codec,

    #[arg(
        long,
        default_value_t = 18,
        value_parser = clap::value_parser!(u32).range(0..=51),
        conflicts_with = "bitrate",
        help = t!("cli.video.crf").to_string(),
    )]
    pub crf: u32,

    #[arg(long, help = t!("cli.video.bitrate").to_string())]
    pub bitrate: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum MsaaLevel {
    Off,
    #[value(name = "2")]
    Two,
    #[value(name = "4")]
    Four,
}

impl MsaaLevel {
    pub fn into_msaa(self) -> Msaa {
        match self {
            MsaaLevel::Off => Msaa::Off,
            MsaaLevel::Two => Msaa::Sample2,
            MsaaLevel::Four => Msaa::Sample4,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Codec {
    #[value(name = "h264")]
    H264,
    #[value(name = "h265")]
    H265,
}

#[derive(Debug, Clone, Parser)]
pub struct GameArgs {
    #[arg(long, default_value_t = 1.0, help = t!("cli.game.note_scale").to_string())]
    pub note_scale: f32,
    #[arg(long, help = t!("cli.game.fc_ap_indicator").to_string())]
    pub fc_ap_indicator: bool,
    #[arg(long, help = t!("cli.game.no_multi_highlight").to_string())]
    pub no_multi_highlight: bool,
    #[arg(long, help = t!("cli.game.hide_hit_effect").to_string())]
    pub hide_hit_effect: bool,
    #[arg(long, help = t!("cli.game.name").to_string())]
    pub name: Option<String>,
    #[arg(long, help = t!("cli.game.level").to_string())]
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
