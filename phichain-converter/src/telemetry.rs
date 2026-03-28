use serde_json::Value;
use std::fs::File;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn track(payload: Value) -> Result<(), std::io::Error> {
    let pid = std::process::id();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let path = std::env::temp_dir().join(format!("phichain-telemetry-{pid}-{timestamp}.json"));
    let file = File::create(&path)?;
    serde_json::to_writer(file, &payload)?;

    let current_exe = std::env::current_exe()?;
    Command::new(current_exe)
        .arg("telemetry")
        .arg("flush")
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}

/// Called by `phichain-converter telemetry flush <path>`
pub fn flush(file: PathBuf) -> Result<(), std::io::Error> {
    todo!();
}
