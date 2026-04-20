//! Phichain offscreen video renderer.
//!
//! Pipeline:
//!   1. Run the game logic headlessly (no window) with `ChartTime` stepped
//!      one video-frame at a time.
//!   2. A 2D camera renders each frame into an offscreen GPU texture.
//!   3. `CapturePlugin` copies that texture back to the CPU and pushes raw
//!      RGBA bytes through a channel.
//!   4. `EncoderPlugin` feeds those bytes into an ffmpeg subprocess, which
//!      encodes the mp4 on the side.

mod args;
mod capture;
mod encoder;
mod respack;
mod utils;

use crate::args::Args;
use crate::capture::{setup_offscreen_target, CapturePlugin};
use crate::encoder::{Encoder, EncoderPlugin};
use crate::respack::RespackPlugin;
use bevy::app::ScheduleRunnerPlugin;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::renderer::RenderDevice;
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;
use bevy_kira_audio::AudioPlugin;
use clap::Parser;
use phichain_assets::AssetsPlugin;
use phichain_chart::project::Project;
use phichain_game::{GameConfig, GamePlugin, GameSet, GameViewport, Paused};
use std::time::{Duration, Instant};

fn main() {
    phichain_assets::setup_assets();

    let args = Args::parse();
    let started = Instant::now();

    App::new()
        .configure_sets(Update, GameSet)
        .insert_resource(ClearColor(Color::srgb_u8(0, 0, 0)))
        .insert_resource(args)
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .set(LogPlugin {
                    filter: "warn,phichain_renderer=info".to_string(),
                    level: bevy::log::Level::DEBUG,
                    ..default()
                })
                // WinitPlugin will panic in environments without a display server.
                .disable::<WinitPlugin>(),
        )
        // Offline rendering: run the loop as fast as possible.
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::ZERO))
        .add_plugins(AudioPlugin)
        .add_plugins(AssetsPlugin)
        .add_plugins(RespackPlugin)
        .add_plugins(GamePlugin)
        .add_plugins(CapturePlugin)
        .add_plugins(EncoderPlugin)
        .add_systems(Startup, setup)
        .run();

    info!(
        "render completed in {:.2}s",
        started.elapsed().as_secs_f64()
    );
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut viewport: ResMut<GameViewport>,
    mut paused: ResMut<Paused>,
    mut game_config: ResMut<GameConfig>,
    render_device: Res<RenderDevice>,
    args: Res<Args>,
) {
    let project = Project::open(args.path.clone().into()).expect("failed to open project");
    let music_duration = utils::audio_duration(
        project
            .path
            .music_path()
            .expect("project is missing its music file"),
    )
    .expect("failed to read audio duration");

    let target = setup_offscreen_target(
        &mut commands,
        &mut images,
        &render_device,
        args.video.width,
        args.video.height,
    );

    commands.spawn((
        Camera2d,
        target,
        // The render target is already sRGB; writing values as-is matches the
        // in-editor preview.
        Tonemapping::None,
        IsDefaultUiCamera,
    ));

    // Stand in for the main-window surrogate values the game code reads.
    viewport.0 = Rect::from_corners(
        Vec2::ZERO,
        Vec2::new(args.video.width as f32, args.video.height as f32),
    );
    paused.0 = false;
    *game_config = args
        .game
        .clone()
        .into_game_config(project.meta.name.clone(), project.meta.level.clone());

    let from = args.from.unwrap_or(0.0);
    let to = args.to.unwrap_or(music_duration);
    commands.insert_resource(Encoder::spawn(&args, from, to));

    phichain_game::loader::load_project(&project, &mut commands).expect("failed to load project");
}
