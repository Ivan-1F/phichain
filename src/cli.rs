use bevy::prelude::*;
use clap::Parser;

/// Phichain - Phigros charting toolchain
#[derive(Parser, Resource, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Load project from this path when launch
    #[arg(short, long)]
    pub project: Option<String>,

    /// The language phichain use
    #[arg(short, long, default_value = "en_us")]
    pub language: String,
}

pub struct CliPlugin;

impl Plugin for CliPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Args::parse());
    }
}
