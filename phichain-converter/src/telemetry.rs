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

const TELEMETRY_FILE_PREFIX: &str = "phichain-telemetry-";

/// Called by `phichain-converter telemetry flush <path>`
pub fn flush(file: PathBuf) -> Result<(), std::io::Error> {
    // Validate: file must be inside the system temp directory
    let file = file.canonicalize()?;
    let temp_dir = std::env::temp_dir().canonicalize()?;
    if !file.starts_with(&temp_dir) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "telemetry file must be in temp directory",
        ));
    }

    // Validate: filename must match expected prefix
    let file_name = file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    if !file_name.starts_with(TELEMETRY_FILE_PREFIX) || !file_name.ends_with(".json") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "unexpected telemetry file name",
        ));
    }

    // Validate: content must be valid JSON
    let content = std::fs::read(&file)?;
    if serde_json::from_slice::<Value>(&content).is_err() {
        let _ = std::fs::remove_file(&file);
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "telemetry file contains invalid JSON",
        ));
    }

    // TODO: POST content to telemetry endpoint

    let _ = std::fs::remove_file(&file);
    Ok(())
}
