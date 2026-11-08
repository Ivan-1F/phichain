use super::locate::Tree;
use super::manifest::{Manifest, Media};
use super::story::Anchor;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiOutput, PrimaryEguiContext};
use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;

/// Width of the media in pixels, whatever the physical size of the window
const MEDIA_WIDTH: u32 = 1920;
/// Frames to wait after the project opens, until fonts, assets and the layout have settled
const OPEN_SETTLE: u32 = 60;

pub enum Command {
    Image {
        id: &'static str,
        anchors: Vec<Anchor>,
    },
}

/// Runs the commands of a story, one per frame once the previous one has settled
#[derive(Resource)]
pub struct Runner {
    out: PathBuf,
    work: PathBuf,
    commands: VecDeque<Command>,
    settle: u32,
    /// Screenshots requested but not yet saved
    pending: usize,
    manifests: Vec<Manifest>,
}

impl Runner {
    pub fn new(out: PathBuf, work: PathBuf, commands: Vec<Command>) -> Self {
        Self {
            out,
            work,
            commands: commands.into(),
            settle: OPEN_SETTLE,
            pending: 0,
            manifests: vec![],
        }
    }

    pub fn project_path(&self) -> PathBuf {
        self.work.join("project")
    }

    fn frame_path(&self, id: &str) -> PathBuf {
        self.work.join("frames").join(format!("{id}.png"))
    }
}

pub fn drive_system(
    mut commands: Commands,
    mut runner: ResMut<Runner>,
    windows: Query<&Window, With<PrimaryWindow>>,
    outputs: Query<&EguiOutput, With<PrimaryEguiContext>>,
) {
    if runner.settle > 0 {
        runner.settle -= 1;
        return;
    }
    let Some(command) = runner.commands.pop_front() else {
        if runner.pending == 0 {
            finish(&runner);
        }
        return;
    };

    match command {
        Command::Image { id, anchors } => {
            let Ok(window) = windows.single() else {
                abort("The primary window is missing");
            };
            let Some(update) = outputs
                .single()
                .ok()
                .and_then(|output| output.platform_output.accesskit_update.clone())
            else {
                abort("The accessibility tree is unavailable");
            };
            let tree = Tree::new(update, window.scale_factor());

            let media_scale = MEDIA_WIDTH as f32 / window.physical_width() as f32;
            let height = (window.physical_height() as f32 * media_scale).round() as u32;
            // logical window coordinates to media pixels
            let factor = window.scale_factor() * media_scale;
            let anchors = anchors
                .iter()
                .map(|anchor| {
                    let Some(rect) = anchor.locator.resolve(&tree) else {
                        abort(&format!(
                            "Anchor `{}` of image `{id}` is not on screen: {}",
                            anchor.name,
                            anchor.locator.describe()
                        ));
                    };
                    let rect = [
                        rect.min.x * factor,
                        rect.min.y * factor,
                        rect.width() * factor,
                        rect.height() * factor,
                    ]
                    .map(f32::round);
                    (anchor.name.clone(), rect)
                })
                .collect::<BTreeMap<_, _>>();

            runner.manifests.push(Manifest {
                version: 1,
                id: id.to_owned(),
                media: Media::Image {
                    src: format!("{id}.webp"),
                    width: MEDIA_WIDTH,
                    height,
                },
                anchors,
            });

            let path = runner.frame_path(id);
            runner.pending += 1;
            commands.spawn(Screenshot::primary_window()).observe(
                move |captured: On<ScreenshotCaptured>, mut runner: ResMut<Runner>| {
                    let saved = captured
                        .image
                        .clone()
                        .try_into_dynamic()
                        .map_err(anyhow::Error::from)
                        .and_then(|image| {
                            std::fs::create_dir_all(
                                path.parent().expect("frames have a directory"),
                            )?;
                            // the alpha channel holds brightness when HDR is enabled
                            image.to_rgb8().save(&path)?;
                            Ok(())
                        });
                    if let Err(error) = saved {
                        abort(&format!("Failed to save {}: {error}", path.display()));
                    }
                    runner.pending -= 1;
                },
            );
        }
    }
}

/// Encodes the frames, writes the manifests and quits
fn finish(runner: &Runner) -> ! {
    if let Err(error) = std::fs::create_dir_all(&runner.out) {
        abort(&format!(
            "Failed to create {}: {error}",
            runner.out.display()
        ));
    }
    for manifest in &runner.manifests {
        let Media::Image { src, width, height } = &manifest.media;
        let status = std::process::Command::new("ffmpeg")
            .args(["-y", "-loglevel", "error", "-i"])
            .arg(runner.frame_path(&manifest.id))
            .args(["-vf", &format!("scale={width}:{height}:flags=lanczos")])
            .args(["-c:v", "libwebp", "-quality", "88"])
            .arg(runner.out.join(src))
            .status();
        match status {
            Ok(status) if status.success() => {}
            Ok(status) => abort(&format!("ffmpeg failed to encode {src}: {status}")),
            Err(error) => abort(&format!("Failed to run ffmpeg: {error}")),
        }

        let path = runner.out.join(format!("{}.json", manifest.id));
        let json = serde_json::to_string_pretty(manifest).expect("manifests serialize");
        if let Err(error) = std::fs::write(&path, json + "\n") {
            abort(&format!("Failed to write {}: {error}", path.display()));
        }
        info!("Captured {}", manifest.id);
    }
    let _ = std::fs::remove_dir_all(&runner.work);
    // a graceful shutdown occasionally hangs on macOS, and nothing is left to clean up
    std::process::exit(0);
}

fn abort(message: &str) -> ! {
    error!("{message}");
    eprintln!("{message}");
    std::process::exit(1);
}
