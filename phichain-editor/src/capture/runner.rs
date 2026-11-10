use super::input::{key_message, key_name, text_message, Inject};
use super::locate::{Locator, Screen, Timelines, Tree};
use super::manifest::{Event, Manifest, Media};
use super::story::Anchor;
use super::FPS;
use crate::file::PendingDialog;
use crate::layout::ui_state::UiState;
use crate::project::Project;
use crate::timing::SeekTo;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::{CursorLeft, CursorMoved, PrimaryWindow};
use bevy_egui::{egui, EguiOutput, PrimaryEguiContext};
use image::RgbImage;
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::BpmList;
use phichain_game::loader::nonblocking::LoadingProject;
use std::collections::{BTreeMap, VecDeque};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Stdio};

/// Frames to wait after the window or a project opens, until fonts, assets and the layout have settled
const OPEN_SETTLE: u32 = 60;
/// Frames for the UI to respond to input, and for the tree to follow
const INPUT_SETTLE: u32 = 6;
/// Video pixels per logical pixel, so a full window of 1280 points becomes 1920 pixels wide
const VIDEO_SCALE: f32 = 1.5;

pub enum Command {
    /// Wait for a number of frames
    Wait(u32),
    /// Move the cursor to the center of the target over a number of frames
    MoveTo(Locator, u32),
    Click,
    Press(KeyCode),
    Type(&'static str),
    /// Pick a file of the working directory in the file dialog that is open
    Choose(&'static str),
    /// Take the cursor out of the window, so nothing shows hover feedback
    Leave,
    /// Move the chart to a beat
    Seek(f32),
    Image(Shot),
    /// Write the interactive nodes that have no name to `<id>.audit.txt`
    Audit(&'static str),
    StartVideo {
        id: &'static str,
        anchors: Vec<Anchor>,
    },
    StopVideo,
    /// Mark where a numbered step of the instructions starts in the video
    Step(u32),
    /// Outline an anchor in the video for a number of frames
    Highlight(Anchor, u32),
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

/// The video being recorded: every frame is captured until it stops
struct Recording {
    id: &'static str,
    anchors: BTreeMap<String, [f32; 4]>,
    events: Vec<Event>,
    /// Frames captured so far
    frames: u64,
}

impl Recording {
    /// Seconds into the video of the frame captured this frame
    fn now(&self) -> f64 {
        (self.frames - 1) as f64 / FPS
    }
}

/// Feeds the frames of a video to ffmpeg in the order they were captured
struct Encoder {
    id: &'static str,
    ffmpeg: Child,
    stdin: Option<ChildStdin>,
    /// Frames captured but not yet fed, waiting for the earlier ones
    waiting: BTreeMap<u64, RgbImage>,
    fed: u64,
    /// The total once recording stopped
    total: Option<u64>,
    last: Option<RgbImage>,
}

impl Encoder {
    fn done(&self) -> bool {
        self.total == Some(self.fed)
    }
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
    release_keys: Vec<KeyboardInput>,
    recording: Option<Recording>,
    encoders: Vec<Encoder>,
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
            release_keys: vec![],
            recording: None,
            encoders: vec![],
            pending: 0,
            manifests: vec![],
        }
    }

    pub fn cursor(&self) -> Option<Vec2> {
        self.cursor
    }

    fn frame_path(&self, id: &str) -> PathBuf {
        self.work.join("frames").join(format!("{id}.png"))
    }

    fn record(&mut self, event: impl FnOnce(f64) -> Event) {
        if let Some(recording) = &mut self.recording {
            let event = event(recording.now());
            recording.events.push(event);
        }
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

    /// Feeds the frames that are next in order, and closes encoders that got all of theirs
    fn feed(&mut self, id: &str, index: u64, image: RgbImage) {
        let encoder = self
            .encoders
            .iter_mut()
            .find(|encoder| encoder.id == id)
            .expect("frames belong to a started video");
        encoder.waiting.insert(index, image);
        while let Some(image) = encoder.waiting.remove(&encoder.fed) {
            let stdin = encoder
                .stdin
                .as_mut()
                .expect("open until every frame is fed");
            if let Err(error) = stdin.write_all(image.as_raw()) {
                abort(&format!(
                    "Failed to feed a frame of `{id}` to ffmpeg: {error}"
                ));
            }
            encoder.fed += 1;
            encoder.last = Some(image);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn drive_system(
    mut commands: Commands,
    mut runner: ResMut<Runner>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    outputs: Query<&EguiOutput, With<PrimaryEguiContext>>,
    ui_state: Res<UiState>,
    project: Option<Res<Project>>,
    loading: Query<(), With<LoadingProject>>,
    dialogs: Query<(Entity, &PendingDialog)>,
    timelines: Option<Timelines>,
    bpm_list: Option<Res<BpmList>>,
    mut seek: MessageWriter<SeekTo>,
    mut inject: Inject,
) {
    let Ok((window_entity, window)) = windows.single() else {
        abort("The primary window is missing");
    };
    let runner = &mut *runner;

    if project.is_some_and(|project| project.is_added()) {
        runner.settle = OPEN_SETTLE;
    }
    // loading takes however many frames it does, which must not shift the story
    if !loading.is_empty() {
        return;
    }

    close_finished_encoders(runner);

    if let Some(recording) = &mut runner.recording {
        let (id, index) = (recording.id, recording.frames);
        recording.frames += 1;
        commands.spawn(Screenshot::primary_window()).observe(
            move |captured: On<ScreenshotCaptured>, mut runner: ResMut<Runner>| {
                match captured.image.clone().try_into_dynamic() {
                    // the alpha channel holds brightness when HDR is enabled
                    Ok(image) => runner.feed(id, index, image.to_rgb8()),
                    Err(error) => abort(&format!("Failed to read a frame of `{id}`: {error}")),
                }
            },
        );
    }

    if runner.release_mouse {
        runner.release_mouse = false;
        inject.mouse(MouseButtonInput {
            button: MouseButton::Left,
            state: ButtonState::Released,
            window: window_entity,
        });
    }
    for message in std::mem::take(&mut runner.release_keys) {
        inject.key(message);
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
        let encoding = runner.recording.is_some() || !runner.encoders.is_empty();
        if runner.pending == 0 && !encoding {
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
    let screen = |tree| Screen {
        tree,
        dock: &ui_state.state,
        timelines: timelines.as_ref(),
    };

    match command {
        Command::Wait(frames) => runner.settle = frames,
        Command::MoveTo(target, frames) => {
            let tree = tree();
            let rect = target.resolve(&screen(&tree)).unwrap_or_else(|error| {
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
            let Some(cursor) = runner.cursor else {
                runner.fail("Clicked with the cursor outside the window", None);
            };
            inject.mouse(MouseButtonInput {
                button: MouseButton::Left,
                state: ButtonState::Pressed,
                window: window_entity,
            });
            runner.release_mouse = true;
            runner.record(|t| Event::Click {
                t,
                at: [cursor.x * VIDEO_SCALE, cursor.y * VIDEO_SCALE].map(f32::round),
            });
            runner.settle = INPUT_SETTLE;
        }
        Command::Press(code) => {
            inject.key(key_message(code, ButtonState::Pressed, window_entity));
            runner
                .release_keys
                .push(key_message(code, ButtonState::Released, window_entity));
            runner.record(|t| Event::Key {
                t,
                keys: key_name(code),
            });
            runner.settle = INPUT_SETTLE;
        }
        Command::Type(text) => {
            for character in text.chars() {
                inject.key(text_message(character, ButtonState::Pressed, window_entity));
                runner.release_keys.push(text_message(
                    character,
                    ButtonState::Released,
                    window_entity,
                ));
            }
            runner.settle = INPUT_SETTLE;
        }
        Command::Choose(file) => {
            let mut open = dialogs.iter();
            let (Some((entity, dialog)), None) = (open.next(), open.next()) else {
                runner.fail(
                    &format!(
                        "Cannot choose `{file}` with {} file dialogs open",
                        dialogs.iter().count()
                    ),
                    None,
                );
            };
            commands.entity(entity).despawn();
            dialog.answer(&mut commands, Some(runner.work.join(file)));
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
        Command::Seek(beat) => {
            let Some(bpm_list) = &bpm_list else {
                runner.fail("Cannot seek before a project opens", None);
            };
            seek.write(SeekTo(bpm_list.time_at(Beat::from(beat))));
            runner.settle = INPUT_SETTLE;
        }
        Command::Image(shot) => {
            let tree = tree();
            capture_image(&mut commands, runner, window, &screen(&tree), shot);
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
        Command::StartVideo { id, anchors } => {
            let tree = tree();
            let screen = screen(&tree);
            let anchors = anchors
                .iter()
                .map(|anchor| {
                    let rect = anchor.locator.resolve(&screen).unwrap_or_else(|error| {
                        runner.fail(
                            &format!(
                                "Cannot find anchor `{}` of video `{id}`: {error}",
                                anchor.name
                            ),
                            Some(&tree),
                        )
                    });
                    (anchor.name.clone(), media_rect(rect, VIDEO_SCALE))
                })
                .collect();
            let encoder = start_encoder(runner, window, id);
            runner.encoders.push(encoder);
            runner.recording = Some(Recording {
                id,
                anchors,
                events: vec![],
                frames: 0,
            });
        }
        Command::StopVideo => {
            let recording = runner.recording.take().expect("a video is being recorded");
            let encoder = runner
                .encoders
                .iter_mut()
                .find(|encoder| encoder.id == recording.id)
                .expect("a started video has an encoder");
            encoder.total = Some(recording.frames);
            let id = recording.id;
            runner.manifests.push(Manifest {
                version: 1,
                id: id.to_owned(),
                media: Media::Video {
                    src: format!("{id}.webm"),
                    poster: format!("{id}.poster.webp"),
                    width: (window.width() * VIDEO_SCALE).round() as u32,
                    height: (window.height() * VIDEO_SCALE).round() as u32,
                    scale: VIDEO_SCALE,
                    duration: recording.frames as f64 / FPS,
                },
                anchors: recording.anchors,
                events: recording.events,
            });
        }
        Command::Step(step) => runner.record(|t| Event::Step { t, step }),
        Command::Highlight(anchor, frames) => {
            let tree = tree();
            let rect = anchor
                .locator
                .resolve(&screen(&tree))
                .unwrap_or_else(|error| {
                    runner.fail(
                        &format!("Cannot find highlight `{}`: {error}", anchor.name),
                        Some(&tree),
                    )
                });
            runner.record(|t| Event::Highlight {
                t,
                anchor: anchor.name,
                rect: media_rect(rect, VIDEO_SCALE),
                until: t + frames as f64 / FPS,
            });
        }
    }
}

/// `[x, y, width, height]` of a logical rect in media pixels
fn media_rect(rect: egui::Rect, scale: f32) -> [f32; 4] {
    let rect = rect * scale;
    [rect.min.x, rect.min.y, rect.width(), rect.height()].map(f32::round)
}

fn start_encoder(runner: &Runner, window: &Window, id: &'static str) -> Encoder {
    let path = runner.work.join(format!("{id}.webm"));
    let spawned = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-loglevel",
            "error",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
        ])
        .args([
            "-s",
            &format!("{}x{}", window.physical_width(), window.physical_height()),
        ])
        .args(["-framerate", &FPS.to_string(), "-i", "-"])
        .args([
            "-vf",
            &format!(
                "scale={}:{}:flags=lanczos",
                (window.width() * VIDEO_SCALE).round(),
                (window.height() * VIDEO_SCALE).round()
            ),
        ])
        .args([
            "-c:v",
            "libvpx-vp9",
            "-b:v",
            "0",
            "-crf",
            "34",
            "-row-mt",
            "1",
        ])
        .args(["-deadline", "good", "-cpu-used", "2", "-pix_fmt", "yuv420p"])
        .arg(&path)
        .stdin(Stdio::piped())
        .spawn();
    let mut ffmpeg = match spawned {
        Ok(ffmpeg) => ffmpeg,
        Err(error) => abort(&format!("Failed to run ffmpeg: {error}")),
    };
    Encoder {
        id,
        stdin: ffmpeg.stdin.take(),
        ffmpeg,
        waiting: BTreeMap::new(),
        fed: 0,
        total: None,
        last: None,
    }
}

/// Lets ffmpeg finish the videos that got all their frames, and keeps their last frame as the poster
fn close_finished_encoders(runner: &mut Runner) {
    let (done, encoding) = std::mem::take(&mut runner.encoders)
        .into_iter()
        .partition::<Vec<_>, _>(Encoder::done);
    runner.encoders = encoding;
    for mut encoder in done {
        drop(encoder.stdin.take());
        match encoder.ffmpeg.wait() {
            Ok(status) if status.success() => {}
            Ok(status) => abort(&format!(
                "ffmpeg failed to encode `{}`: {status}",
                encoder.id
            )),
            Err(error) => abort(&format!("Failed to wait for ffmpeg: {error}")),
        }
        let poster = runner.frame_path(&format!("{}.poster", encoder.id));
        let saved = std::fs::create_dir_all(poster.parent().expect("frames have a directory"))
            .map_err(anyhow::Error::from)
            .and_then(|_| {
                let last = encoder.last.as_ref().expect("a video has frames");
                Ok(last.save(&poster)?)
            });
        if let Err(error) = saved {
            abort(&format!(
                "Failed to save the poster of `{}`: {error}",
                encoder.id
            ));
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
    let scale_factor = window.scale_factor();
    let anchors = shot
        .anchors
        .iter()
        .map(|anchor| {
            let rect = anchor.locator.resolve(screen).unwrap_or_else(|error| {
                runner.fail(
                    &format!(
                        "Cannot find anchor `{}` of image `{}`: {error}",
                        anchor.name, shot.id
                    ),
                    Some(screen.tree),
                )
            });
            // images keep every physical pixel, so they stay sharp at their natural size
            (anchor.name.clone(), media_rect(rect, scale_factor))
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
        events: vec![],
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

/// Encodes the images, moves the videos out, writes the manifests and quits
fn finish(runner: &Runner) -> ! {
    if let Err(error) = std::fs::create_dir_all(&runner.out) {
        abort(&format!(
            "Failed to create {}: {error}",
            runner.out.display()
        ));
    }
    let webp = |input: PathBuf, output: &str, size: Option<(u32, u32)>| {
        let mut ffmpeg = std::process::Command::new("ffmpeg");
        ffmpeg.args(["-y", "-loglevel", "error", "-i"]).arg(input);
        if let Some((width, height)) = size {
            ffmpeg.args(["-vf", &format!("scale={width}:{height}:flags=lanczos")]);
        }
        let status = ffmpeg
            .args(["-c:v", "libwebp", "-quality", "88"])
            .arg(runner.out.join(output))
            .status();
        match status {
            Ok(status) if status.success() => {}
            Ok(status) => abort(&format!("ffmpeg failed to encode {output}: {status}")),
            Err(error) => abort(&format!("Failed to run ffmpeg: {error}")),
        }
    };
    for manifest in &runner.manifests {
        match &manifest.media {
            Media::Image { src, .. } => webp(runner.frame_path(&manifest.id), src, None),
            Media::Video {
                src,
                poster,
                width,
                height,
                ..
            } => {
                let poster_frame = runner.frame_path(&format!("{}.poster", manifest.id));
                webp(poster_frame, poster, Some((*width, *height)));
                if let Err(error) = std::fs::copy(runner.work.join(src), runner.out.join(src)) {
                    abort(&format!("Failed to move {src}: {error}"));
                }
            }
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
