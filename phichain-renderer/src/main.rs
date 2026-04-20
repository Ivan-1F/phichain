//! Phichain offscreen video renderer.
//!
//! Pipeline:
//!   1. Game logic runs headlessly with `ChartTime` stepped one video-frame
//!      at a time.
//!   2. A 2D camera renders each frame into an offscreen GPU texture.
//!   3. Bevy's built-in `Readback` component copies that texture back to the
//!      CPU each frame, firing `ReadbackComplete`.
//!   4. The observer in `encoder` feeds the bytes into an ffmpeg subprocess
//!      which encodes the mp4 on the side.

mod args;
mod encoder;
mod respack;
mod utils;

use crate::args::Args;
use crate::encoder::{on_frame_ready, Encoder};
use crate::respack::RespackPlugin;
use bevy::app::ScheduleRunnerPlugin;
use bevy::camera::RenderTarget;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::gpu_readback::Readback;
use bevy::render::render_resource::{TextureFormat, TextureUsages};
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
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .set(LogPlugin {
                    // Silence the shutdown-time readback-channel warning; we
                    // intentionally exit with a few readbacks still in flight.
                    filter: "warn,phichain_renderer=info,bevy_render::gpu_readback=error"
                        .to_string(),
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

    // Offscreen GPU texture the camera renders into; Readback copies it out each frame.
    let mut target = Image::new_target_texture(
        args.video.width,
        args.video.height,
        TextureFormat::Rgba8UnormSrgb,
        None,
    );
    target.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    let target_handle = images.add(target);

    commands.spawn((
        Camera2d,
        RenderTarget::Image(target_handle.clone().into()),
        // The target is already sRGB; tonemapping would double-encode.
        Tonemapping::None,
        IsDefaultUiCamera,
        args.video.msaa.into_msaa(),
    ));

    commands
        .spawn(Readback::texture(target_handle))
        .observe(on_frame_ready);

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
