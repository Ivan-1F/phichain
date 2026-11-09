use super::input::Inject;
use super::locate::{Locator, Screen, Tree};
use super::manifest::{Manifest, Media};
use super::story::Anchor;
use crate::layout::ui_state::UiState;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::{CursorLeft, CursorMoved, PrimaryWindow};
use bevy_egui::{EguiOutput, PrimaryEguiContext};
use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;

/// Frames to wait after the project opens, until fonts, assets and the layout have settled
const OPEN_SETTLE: u32 = 60;
/// Frames for the UI to respond to a click or the pointer leaving, and for the tree to follow
const INPUT_SETTLE: u32 = 6;

pub enum Command {
    /// Move the cursor to the center of the target over a number of frames
    MoveTo(Locator, u32),
    Click,
    /// Take the cursor out of the window, so nothing shows hover feedback
    Leave,
    Image(Shot),
    /// Write the interactive nodes that have no name to `<id>.audit.txt`
    Audit(&'static str),
}

/// An image of the whole window, so readers see things where they are in their own editor
pub struct Shot {
    id: &'static str,
    anchors: Vec<Anchor>,
}

impl Shot {
    pub fn new(id: &'static str) -> Self {
        Self {
            id,
            anchors: vec![],
        }
    }

    pub fn anchor(&mut self, anchor: Anchor) -> &mut Self {
        self.anchors.push(anchor);
        self
    }

    pub fn anchors(&mut self, anchors: impl IntoIterator<Item = Anchor>) -> &mut Self {
        self.anchors.extend(anchors);
        self
    }
}

struct Move {
    from: Vec2,
    to: Vec2,
    frame: u32,
    total: u32,
}

/// Runs the commands of a story, one per frame once the previous one has settled
#[derive(Resource)]
pub struct Runner {
    out: PathBuf,
    work: PathBuf,
    commands: VecDeque<Command>,
    settle: u32,
    /// The story's cursor in logical window coordinates, `None` while outside the window
    cursor: Option<Vec2>,
    moving: Option<Move>,
    release_mouse: bool,
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
            cursor: None,
            moving: None,
            release_mouse: false,
            pending: 0,
            manifests: vec![],
        }
    }

    pub fn project_path(&self) -> PathBuf {
        self.work.join("project")
    }

    pub fn cursor(&self) -> Option<Vec2> {
        self.cursor
    }

    fn frame_path(&self, id: &str) -> PathBuf {
        self.work.join("frames").join(format!("{id}.png"))
    }

    /// Stops the run, keeping the working directory and the tree for a look at what went wrong
    fn fail(&self, message: &str, tree: Option<&Tree>) -> ! {
        if let Some(tree) = tree {
            let path = self.work.join("tree.txt");
            if std::fs::write(&path, tree.dump()).is_ok() {
                abort(&format!(
                    "{message}\nThe accessibility tree is in {}",
                    path.display()
                ));
            }
        }
        abort(message);
    }
}

pub fn drive_system(
    mut commands: Commands,
    mut runner: ResMut<Runner>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    outputs: Query<&EguiOutput, With<PrimaryEguiContext>>,
    ui_state: Res<UiState>,
    mut inject: Inject,
) {
    let Ok((window_entity, window)) = windows.single() else {
        abort("The primary window is missing");
    };
    let runner = &mut *runner;

    if runner.release_mouse {
        runner.release_mouse = false;
        inject.mouse(MouseButtonInput {
            button: MouseButton::Left,
            state: ButtonState::Released,
            window: window_entity,
        });
    }

    if let Some(moving) = &mut runner.moving {
        moving.frame += 1;
        let t = moving.frame as f32 / moving.total as f32;
        // ease in and out, so the cursor speeds up and settles like a hand would
        let eased = if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        };
        let position = moving.from.lerp(moving.to, eased);
        if moving.frame >= moving.total {
            runner.moving = None;
        }
        runner.cursor = Some(position);
        inject.cursor(CursorMoved {
            window: window_entity,
            position,
            delta: None,
        });
        return;
    }

    if runner.settle > 0 {
        runner.settle -= 1;
        return;
    }

    let Some(command) = runner.commands.pop_front() else {
        if runner.pending == 0 {
            finish(runner);
        }
        return;
    };

    let tree = || {
        let Some(update) = outputs
            .single()
            .ok()
            .and_then(|output| output.platform_output.accesskit_update.clone())
        else {
            abort("The accessibility tree is unavailable");
        };
        Tree::new(update, window.scale_factor())
    };

    match command {
        Command::MoveTo(target, frames) => {
            let tree = tree();
            let screen = Screen {
                tree: &tree,
                dock: &ui_state.state,
            };
            let rect = target.resolve(&screen).unwrap_or_else(|error| {
                runner.fail(&format!("Cannot find the target: {error}"), Some(&tree))
            });
            let to = Vec2::new(rect.center().x, rect.center().y);
            runner.moving = Some(Move {
                // coming from outside, the cursor enters right at the target
                from: runner.cursor.unwrap_or(to),
                to,
                frame: 0,
                total: frames,
            });
        }
        Command::Click => {
            if runner.cursor.is_none() {
                runner.fail("Clicked with the cursor outside the window", None);
            }
            inject.mouse(MouseButtonInput {
                button: MouseButton::Left,
                state: ButtonState::Pressed,
                window: window_entity,
            });
            runner.release_mouse = true;
            runner.settle = INPUT_SETTLE;
        }
        Command::Leave => {
            if runner.cursor.take().is_some() {
                inject.left(CursorLeft {
                    window: window_entity,
                });
                runner.settle = INPUT_SETTLE;
            }
        }
        Command::Image(shot) => {
            let tree = tree();
            let screen = Screen {
                tree: &tree,
                dock: &ui_state.state,
            };
            capture_image(&mut commands, runner, window, &screen, shot);
        }
        Command::Audit(id) => {
            let audit = tree().audit();
            let path = runner.out.join(format!("{id}.audit.txt"));
            let written = std::fs::create_dir_all(&runner.out)
                .and_then(|_| std::fs::write(&path, audit.report()));
            if let Err(error) = written {
                runner.fail(
                    &format!("Failed to write {}: {error}", path.display()),
                    None,
                );
            }
            info!(
                "{} widgets without a name, written to {}",
                audit.widgets.len(),
                path.display()
            );
        }
    }
}

fn capture_image(
    commands: &mut Commands,
    runner: &mut Runner,
    window: &Window,
    screen: &Screen,
    shot: Shot,
) {
    let locate = |locator: &Locator, what: &str| {
        locator.resolve(screen).unwrap_or_else(|error| {
            runner.fail(
                &format!("Cannot find {what} of image `{}`: {error}", shot.id),
                Some(screen.tree),
            )
        })
    };

    let scale_factor = window.scale_factor();
    let anchors = shot
        .anchors
        .iter()
        .map(|anchor| {
            let rect = locate(&anchor.locator, &format!("anchor `{}`", anchor.name));
            // images keep every physical pixel, so they stay sharp at their natural size
            let rect = rect * scale_factor;
            let rect = [rect.min.x, rect.min.y, rect.width(), rect.height()].map(f32::round);
            (anchor.name.clone(), rect)
        })
        .collect::<BTreeMap<_, _>>();

    runner.manifests.push(Manifest {
        version: 1,
        id: shot.id.to_owned(),
        media: Media::Image {
            src: format!("{}.webp", shot.id),
            width: window.physical_width(),
            height: window.physical_height(),
            scale: scale_factor,
        },
        anchors,
    });

    let path = runner.frame_path(shot.id);
    runner.pending += 1;
    commands.spawn(Screenshot::primary_window()).observe(
        move |captured: On<ScreenshotCaptured>, mut runner: ResMut<Runner>| {
            let saved = captured
                .image
                .clone()
                .try_into_dynamic()
                .map_err(anyhow::Error::from)
                .and_then(|image| {
                    std::fs::create_dir_all(path.parent().expect("frames have a directory"))?;
                    // the alpha channel holds brightness when HDR is enabled
                    image.to_rgb8().save(&path)?;
                    Ok(())
                });
            if let Err(error) = saved {
                runner.fail(&format!("Failed to save {}: {error}", path.display()), None);
            }
            runner.pending -= 1;
        },
    );
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
        let Media::Image { src, .. } = &manifest.media;
        let status = std::process::Command::new("ffmpeg")
            .args(["-y", "-loglevel", "error", "-i"])
            .arg(runner.frame_path(&manifest.id))
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
