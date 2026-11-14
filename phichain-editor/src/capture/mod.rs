//! Capture mode: runs a built-in story that drives the editor and saves media for the documentation
//!
//! Started with `--capture <story>`. Every run uses a fresh working directory, a fixed time step
//! and fixed randomness, so the same story produces the same media.

mod input;
mod locate;
mod manifest;
mod runner;
mod stories;
mod story;

use crate::cli::Args;
use crate::file::NoSystemDialogs;
use crate::misc::WorkingDirectory;
use crate::settings::EditorSettings;
use crate::timing::NoAudioSync;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts, EguiPostUpdateSet, EguiPreUpdateSet, EguiPrimaryContextPass};
use phichain_game::loader::nonblocking::LoadingProject;
use phichain_game::HitEffectRng;
use runner::Runner;
use std::path::{Path, PathBuf};
use std::time::Duration;

const FPS: f64 = 60.0;
const FRAME: Duration = Duration::from_nanos((1e9 / FPS) as u64);
/// Logical size of the window, the same for every story
const WINDOW_SIZE: (u32, u32) = (1280, 800);
const SCALE_FACTOR: f32 = 2.0;
/// Length of the silent music of the demo project
const MUSIC_SECONDS: u32 = 120;

/// What the working directory starts with, for stories to pick in file dialogs
pub const MUSIC: &str = "music.wav";
pub const ILLUSTRATION: &str = "illustration.png";
/// An empty folder to create the project in
pub const PROJECT: &str = "project";

#[derive(clap::Args, Debug)]
pub struct CaptureArgs {
    /// Run a capture story and write its media and manifests into the output directory
    #[arg(long, value_name = "STORY", requires_all = ["capture_language", "capture_out"])]
    pub capture: Option<String>,
    /// Editor language while capturing, such as zh_cn
    #[arg(long, value_name = "LANGUAGE", requires = "capture")]
    pub capture_language: Option<String>,
    /// Directory to write the media and manifests into
    #[arg(long, value_name = "DIR", requires = "capture")]
    pub capture_out: Option<PathBuf>,
    /// Print the names of the capture stories, one per line
    #[arg(long, conflicts_with = "capture")]
    pub capture_list: bool,
}

pub struct CapturePlugin;

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        let args = &app.world().resource::<Args>().capture;
        if args.capture_list {
            for name in stories::names() {
                println!("{name}");
            }
            std::process::exit(0);
        }
        let Some(name) = args.capture.clone() else {
            return;
        };
        let (Some(language), Some(out)) = (args.capture_language.clone(), args.capture_out.clone())
        else {
            unreachable!("clap requires them together with --capture");
        };

        let Some(story) = stories::find(&name) else {
            eprintln!(
                "Unknown capture story `{name}`, available stories: {}",
                stories::names().join(", ")
            );
            std::process::exit(2);
        };
        if !rust_i18n::available_locales!().contains(&language.as_str()) {
            eprintln!("Unknown language `{language}`");
            std::process::exit(2);
        }

        let work = std::env::temp_dir().join(format!("phichain-capture-{}", std::process::id()));
        if let Err(error) = prepare(&work, &language) {
            eprintln!("Failed to prepare the capture workspace: {error:?}");
            std::process::exit(1);
        }

        app.insert_resource(WorkingDirectory(work.clone()))
            .insert_resource(TimeUpdateStrategy::ManualDuration(FRAME))
            .insert_resource(NoAudioSync)
            .insert_resource(NoSystemDialogs)
            .insert_resource(HitEffectRng::seeded(0))
            .insert_resource(Runner::new(out, work, story.commands))
            .add_systems(Startup, setup_system)
            .add_systems(
                First,
                (
                    input::discard_real_input_system,
                    clock_system,
                    runner::drive_system,
                    input::place_cursor_system,
                )
                    .chain(),
            )
            .add_systems(
                PreUpdate,
                input::focus_window_system
                    .after(EguiPreUpdateSet::ProcessInput)
                    .before(EguiPreUpdateSet::BeginPass),
            )
            .add_systems(
                PostUpdate,
                input::lift_cursor_system.after(EguiPostUpdateSet::ProcessOutput),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (enable_accesskit_system, input::draw_cursor_system),
            );
    }
}

/// Writes the settings and the files of the demo project into a fresh working directory
fn prepare(work: &Path, language: &str) -> anyhow::Result<()> {
    if work.exists() {
        std::fs::remove_dir_all(work)?;
    }
    let config = work.join("config");
    std::fs::create_dir_all(&config)?;

    let mut settings = EditorSettings::default();
    settings.general.language = language.to_owned();
    settings.general.send_telemetry = false;
    settings.audio.music_volume = 0.0;
    settings.audio.hit_sound_volume = 0.0;
    settings.audio.metronome_volume = 0.0;
    std::fs::write(
        config.join("settings.yml"),
        serde_yaml::to_string(&settings)?,
    )?;

    write_silence(&work.join(MUSIC), MUSIC_SECONDS)?;
    write_illustration(&work.join(ILLUSTRATION))?;
    std::fs::create_dir_all(work.join(PROJECT))?;

    Ok(())
}

fn write_silence(path: &Path, seconds: u32) -> anyhow::Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 8000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)?;
    for _ in 0..spec.sample_rate * seconds {
        writer.write_sample(0i16)?;
    }
    writer.finalize()?;
    Ok(())
}

/// A soft diagonal gradient, plain enough to stay behind the chart
fn write_illustration(path: &Path) -> anyhow::Result<()> {
    let (width, height) = (1920, 1080);
    let from = [40.0, 62.0, 110.0];
    let to = [120.0, 70.0, 140.0];
    let image = image::RgbImage::from_fn(width, height, |x, y| {
        let t = (x as f32 / width as f32 + y as f32 / height as f32) / 2.0;
        image::Rgb(std::array::from_fn(|i| {
            (from[i] + (to[i] - from[i]) * t).round() as u8
        }))
    });
    image.save(path)?;
    Ok(())
}

fn setup_system(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = windows.single_mut() {
        window
            .resolution
            .set(WINDOW_SIZE.0 as f32, WINDOW_SIZE.1 as f32);
        window
            .resolution
            .set_scale_factor_override(Some(SCALE_FACTOR));
    }
}

/// Holds the clock while a project loads
///
/// Chart time derives from the time since launch, so it must not count the frames loading happens to take.
fn clock_system(
    mut strategy: ResMut<TimeUpdateStrategy>,
    loading: Query<(), With<LoadingProject>>,
) {
    let step = if loading.is_empty() {
        FRAME
    } else {
        Duration::ZERO
    };
    *strategy = TimeUpdateStrategy::ManualDuration(step);
}

fn enable_accesskit_system(mut contexts: EguiContexts) -> Result {
    contexts.ctx_mut()?.enable_accesskit();
    Ok(())
}
