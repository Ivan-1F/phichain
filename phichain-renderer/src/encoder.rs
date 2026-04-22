//! ffmpeg subprocess management and per-frame writing.
//!
//! A child `ffmpeg` reads raw RGBA bytes from stdin at a fixed size/framerate
//! and produces an h264 mp4. We drive `ChartTime` one video-frame at a time;
//! the game code renders a matching frame; Bevy's built-in `Readback` copies
//! that frame out of the GPU and fires `ReadbackComplete`, which we observe
//! here to pipe the bytes into ffmpeg.

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::render::gpu_readback::ReadbackComplete;
use bevy::render::renderer::RenderDevice;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use phichain_game::ChartTime;
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::time::Instant;
use tempfile::NamedTempFile;

use crate::args::{Args, Codec};

/// Frames rendered before we start writing, giving the GPU time to warm up
/// (shader compilation, first-frame cache misses). The earliest frames are
/// often transparent or blocky and would show up as garbage.
const WARMUP_FRAMES: u32 = 40;

#[derive(Resource)]
pub struct Encoder {
    ffmpeg: Child,
    width: u32,
    height: u32,
    fps: u32,
    from: f32,
    to: f32,

    warmup_remaining: u32,
    frames_written: u32,

    start: Instant,
    progress: ProgressBar,

    // Keep the WAV alive until ffmpeg exits.
    _audio: NamedTempFile,
}

impl Encoder {
    pub fn spawn(args: &Args, from: f32, to: f32, audio: NamedTempFile) -> Self {
        let (width, height, fps) = (args.video.width, args.video.height, args.video.fps);
        let total_frames = (fps as f32 * (to - from)) as u32;

        let mut cmd = Command::new("ffmpeg");
        cmd.args(["-y"])
            .args(["-f", "rawvideo", "-pix_fmt", "rgba"])
            .args(["-s", &format!("{width}x{height}")])
            .args(["-framerate", &fps.to_string()])
            .args(["-i", "-"])
            .arg("-i")
            .arg(audio.path());

        let encoder = pick_encoder(args.video.codec, args.video.hwaccel);
        cmd.args(["-c:v", encoder]);
        for arg in build_quality_args(args, encoder) {
            cmd.arg(arg);
        }

        // alimiter catches additive overshoots from overlapping hit sounds.
        cmd.args(["-c:a", "aac", "-b:a", "192k"])
            .args(["-af", "alimiter=limit=0.95:level=disabled"])
            .args(["-map", "0:v:0", "-map", "1:a:0"])
            .arg("-shortest");

        cmd.arg(&args.output)
            .stdin(Stdio::piped())
            .stderr(Stdio::null());
        let ffmpeg = cmd
            .spawn()
            .expect("failed to spawn ffmpeg (is it on PATH?)");

        let progress = build_progress_bar(total_frames as u64, fps);

        Self {
            ffmpeg,
            width,
            height,
            fps,
            from,
            to,
            warmup_remaining: WARMUP_FRAMES,
            frames_written: 0,
            start: Instant::now(),
            progress,
            _audio: audio,
        }
    }

    fn next_chart_time(&self) -> f32 {
        self.from + self.frames_written as f32 / self.fps as f32
    }

    fn done(&self) -> bool {
        self.next_chart_time() >= self.to
    }
}

/// Observer fired by Bevy's `GpuReadbackPlugin` each time a frame has been
/// copied back from the GPU.
pub fn on_frame_ready(
    event: On<ReadbackComplete>,
    mut enc: ResMut<Encoder>,
    mut chart_time: ResMut<ChartTime>,
    mut exit: MessageWriter<AppExit>,
) {
    chart_time.0 = enc.next_chart_time();

    if enc.warmup_remaining > 0 {
        enc.warmup_remaining -= 1;
        return;
    }

    let (width, height) = (enc.width, enc.height);
    let pixels = unpad_rows(&event.data, width, height);
    let stdin = enc
        .ffmpeg
        .stdin
        .as_mut()
        .expect("ffmpeg stdin was closed early");
    stdin
        .write_all(&pixels)
        .expect("failed to write frame to ffmpeg");

    enc.frames_written += 1;
    enc.progress.set_position(enc.frames_written as u64);

    if enc.done() {
        enc.progress.finish_and_clear();
        let elapsed = enc.start.elapsed().as_secs_f32();
        let avg_fps = enc.frames_written as f32 / elapsed;
        let realtime = avg_fps / enc.fps as f32;
        info!(
            "encoded {} frames in {:.2}s (avg {:.0} fps, {:.2}x realtime)",
            enc.frames_written, elapsed, avg_fps, realtime,
        );
        // Closing stdin signals EOF; ffmpeg finalizes the file on its own.
        drop(enc.ffmpeg.stdin.take());
        enc.ffmpeg
            .wait()
            .expect("ffmpeg exited with a non-zero status");
        exit.write(AppExit::Success);
    }
}

/// Pick the ffmpeg encoder name for a `(codec, hardware-accel)` combination.
/// Hardware encoder selection is best-effort per-platform.
fn pick_encoder(codec: Codec, hwaccel: bool) -> &'static str {
    match (codec, hwaccel) {
        (Codec::H264, false) => "libx264",
        (Codec::H265, false) => "libx265",
        (Codec::H264, true) => {
            if cfg!(target_os = "macos") {
                "h264_videotoolbox"
            } else if cfg!(target_os = "windows") {
                "h264_qsv"
            } else {
                "h264_nvenc"
            }
        }
        (Codec::H265, true) => {
            if cfg!(target_os = "macos") {
                "hevc_videotoolbox"
            } else if cfg!(target_os = "windows") {
                "hevc_qsv"
            } else {
                "hevc_nvenc"
            }
        }
    }
}

/// Translate our `--bitrate` / `--crf` flags into ffmpeg args for the chosen encoder.
/// Each encoder family uses a different quality knob.
fn build_quality_args(args: &Args, encoder: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(rate) = &args.video.bitrate {
        out.push("-b:v".into());
        out.push(rate.clone());
    } else {
        let crf = args.video.crf;
        let (flag, value) = if encoder.ends_with("videotoolbox") {
            // videotoolbox uses -q:v 1..100 (higher = better). Approximate map.
            let q = (100i32 - crf as i32 * 2).clamp(1, 100);
            ("-q:v", q.to_string())
        } else if encoder.ends_with("nvenc") {
            ("-cq", crf.to_string())
        } else if encoder.ends_with("qsv") {
            ("-global_quality", crf.to_string())
        } else {
            // libx264 / libx265
            ("-crf", crf.to_string())
        };
        out.push(flag.into());
        out.push(value);
    }
    out
}

/// Strip the per-row padding wgpu adds when copying a texture into a buffer
/// (rows are aligned to 256 bytes).
fn unpad_rows(bytes: &[u8], width: u32, height: u32) -> Vec<u8> {
    let row = width as usize * 4;
    let padded = RenderDevice::align_copy_bytes_per_row(row);
    if row == padded {
        return bytes.to_vec();
    }
    let mut out = Vec::with_capacity(row * height as usize);
    for chunk in bytes.chunks_exact(padded).take(height as usize) {
        out.extend_from_slice(&chunk[..row]);
    }
    out
}

/// Build the in-place progress bar shown during encoding.
/// The `fps` parameter is the target output framerate, used to compute the realtime multiplier.
fn build_progress_bar(total_frames: u64, fps: u32) -> ProgressBar {
    let target_fps = fps as f64;
    let template = "[{elapsed_precise}] [{bar:40.cyan/blue}] \
        {pos}/{len} ({percent:>3}%) {fps} ({rt}) eta {eta:>4}";
    let style = ProgressStyle::with_template(template)
        .expect("progress bar template is valid")
        .progress_chars("=> ")
        .with_key("fps", |s: &ProgressState, w: &mut dyn std::fmt::Write| {
            let _ = write!(w, "{:>3.0} fps", s.per_sec());
        })
        .with_key(
            "rt",
            move |s: &ProgressState, w: &mut dyn std::fmt::Write| {
                let _ = write!(w, "{:.2}x", s.per_sec() / target_fps);
            },
        );
    ProgressBar::new(total_frames).with_style(style)
}
