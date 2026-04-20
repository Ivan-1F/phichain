//! Mix music + hit sounds into a temp WAV consumed by the encoder.

use anyhow::{bail, Context, Result};
use bevy::log::info;
use hound::{SampleFormat, WavSpec, WavWriter};
use phichain_assets::{builtin_respack_dir, load_respack, LoadedAudio};
use phichain_chart::bpm_list::BpmList;
use phichain_chart::migration::migrate;
use phichain_chart::note::NoteKind;
use phichain_chart::project::Project;
use phichain_chart::serialization::{PhichainChart, SerializedLine};
use serde_json::Value;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;
use tempfile::NamedTempFile;

const SAMPLE_RATE: u32 = 48_000;
const CHANNELS: u16 = 2;

/// Returned tempfile must outlive the ffmpeg process that reads it.
pub fn render_audio_track(
    project: &Project,
    respack: Option<&Path>,
    from: f32,
    to: f32,
) -> Result<NamedTempFile> {
    assert!(to > from, "--to must be greater than --from");

    let started = Instant::now();

    let chart = read_chart(project).context("read chart")?;
    let offset_secs = chart.offset.0 / 1000.0;
    let notes = collect_notes(&chart, from, to);

    let music_path = project
        .path
        .music_path()
        .context("project is missing its music file")?;
    let music_bytes = std::fs::read(&music_path)
        .with_context(|| format!("read music file {}", music_path.display()))?;
    let music = decode_pcm(&music_bytes).context("decode music")?;
    let sfx = load_hit_sounds(respack).context("load hit sounds")?;

    let out_samples =
        ((to - from) as f64 * SAMPLE_RATE as f64).round() as usize * CHANNELS as usize;
    let mut buf = vec![0.0f32; out_samples];

    overlay_music(&mut buf, &music, from + offset_secs);
    accumulate(&mut buf, &sfx.tap, &notes.taps, from);
    accumulate(&mut buf, &sfx.drag, &notes.drags, from);
    accumulate(&mut buf, &sfx.flick, &notes.flicks, from);

    let total_notes = notes.taps.len() + notes.drags.len() + notes.flicks.len();
    let temp = write_wav(&buf)?;

    info!(
        "audio track ready: {} notes over {:.2}s mixed in {:.2}s",
        total_notes,
        to - from,
        started.elapsed().as_secs_f32()
    );

    Ok(temp)
}

fn read_chart(project: &Project) -> Result<PhichainChart> {
    let file = File::open(project.path.chart_path())?;
    let raw: Value = serde_json::from_reader(file)?;
    let migrated = migrate(&raw)?;
    Ok(serde_json::from_value(migrated)?)
}

#[derive(Default)]
struct NoteTimes {
    // Hold onsets share the tap sfx, same as phichain-editor.
    taps: Vec<f32>,
    drags: Vec<f32>,
    flicks: Vec<f32>,
}

fn collect_notes(chart: &PhichainChart, from: f32, to: f32) -> NoteTimes {
    let mut out = NoteTimes::default();
    for line in &chart.lines {
        walk_line(line, &chart.bpm_list, from, to, &mut out);
    }
    out
}

fn walk_line(line: &SerializedLine, bpm: &BpmList, from: f32, to: f32, out: &mut NoteTimes) {
    for note in &line.notes {
        let t = bpm.time_at(note.beat);
        if t < from || t >= to {
            continue;
        }
        match note.kind {
            NoteKind::Tap | NoteKind::Hold { .. } => out.taps.push(t),
            NoteKind::Drag => out.drags.push(t),
            NoteKind::Flick => out.flicks.push(t),
        }
    }
    for child in &line.children {
        walk_line(child, bpm, from, to, out);
    }
}

struct HitSounds {
    tap: Vec<f32>,
    drag: Vec<f32>,
    flick: Vec<f32>,
}

fn load_hit_sounds(respack: Option<&Path>) -> Result<HitSounds> {
    let pack = match respack {
        Some(path) => {
            load_respack(path).with_context(|| format!("load respack at {}", path.display()))?
        }
        None => load_respack(&builtin_respack_dir()).context("load built-in respack")?,
    };
    let LoadedAudio { tap, drag, flick } = pack.audio;
    Ok(HitSounds {
        tap: decode_pcm(&tap)?,
        drag: decode_pcm(&drag)?,
        flick: decode_pcm(&flick)?,
    })
}

fn decode_pcm(bytes: &[u8]) -> Result<Vec<f32>> {
    let mut child = Command::new("ffmpeg")
        .args(["-v", "error", "-i", "-"])
        .args(["-f", "f32le", "-ar", "48000", "-ac", "2", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn ffmpeg")?;

    // Close stdin before waiting; otherwise ffmpeg blocks on EOF.
    {
        let mut stdin = child.stdin.take().expect("piped stdin");
        stdin.write_all(bytes)?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        bail!(
            "ffmpeg failed decoding audio: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(pcm_bytes_to_f32(&output.stdout))
}

fn pcm_bytes_to_f32(raw: &[u8]) -> Vec<f32> {
    raw.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn overlay_music(out: &mut [f32], music: &[f32], music_start_secs: f32) {
    let offset_samples =
        (music_start_secs * SAMPLE_RATE as f32).round() as isize * CHANNELS as isize;
    let (src_start, dst_start) = if offset_samples >= 0 {
        (offset_samples as usize, 0)
    } else {
        (0, (-offset_samples) as usize)
    };
    if src_start >= music.len() || dst_start >= out.len() {
        return;
    }
    let copy_len = (music.len() - src_start).min(out.len() - dst_start);
    out[dst_start..dst_start + copy_len].copy_from_slice(&music[src_start..src_start + copy_len]);
}

fn accumulate(out: &mut [f32], sfx: &[f32], times: &[f32], from: f32) {
    let channels = CHANNELS as usize;
    for &t in times {
        let frame = ((t - from) * SAMPLE_RATE as f32).round() as isize;
        if frame < 0 {
            continue;
        }
        let start = frame as usize * channels;
        if start >= out.len() {
            continue;
        }
        let len = sfx.len().min(out.len() - start);
        for (d, s) in out[start..start + len].iter_mut().zip(&sfx[..len]) {
            *d += *s;
        }
    }
}

fn write_wav(samples: &[f32]) -> Result<NamedTempFile> {
    let temp = tempfile::Builder::new()
        .prefix("phichain_audio_")
        .suffix(".wav")
        .tempfile()?;
    let file = temp.reopen()?;
    let spec = WavSpec {
        channels: CHANNELS,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };
    let mut writer = WavWriter::new(BufWriter::new(file), spec)?;
    for &s in samples {
        writer.write_sample(s)?;
    }
    writer.finalize()?;
    Ok(temp)
}
