use anyhow::Context;
use std::path::PathBuf;
use std::process::Command;

/// Get the duration of a audio file in seconds using ffprobe
pub fn audio_duration(path: PathBuf) -> anyhow::Result<f32> {
    let output = Command::new("ffprobe")
        .arg("-i")
        .arg(path)
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-v")
        .arg("quiet")
        .arg("-of")
        .arg("csv=p=0")
        .output()?;

    String::from_utf8(output.stdout.trim_ascii().into())?
        .parse::<f32>()
        .context("Failed to parse ffprobe output")
}
