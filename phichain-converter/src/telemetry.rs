use phichain_telemetry::payload::{PhichainMeta, TelemetryPayload};
use serde_json::Value;
use std::fs::File;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn track(event_type: &str, metadata: Value) -> Result<(), std::io::Error> {
    let payload = TelemetryPayload::builder()
        .reporter("phichain-converter")
        .event_type(event_type)
        .maybe_device_id(phichain_telemetry::device::get_device_id())
        .phichain(PhichainMeta::new(
            env!("CARGO_PKG_VERSION"),
            cfg!(debug_assertions),
        ))
        .metadata(metadata)
        .build();

    if phichain_telemetry::env::telemetry_debug() {
        eprintln!("[telemetry] {}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    let pid = std::process::id();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
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
    let payload: Value = serde_json::from_slice(&content).map_err(|_| {
        let _ = std::fs::remove_file(&file);
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "telemetry file contains invalid JSON",
        )
    })?;

    // wrap in array to match the batch format
    let batch = serde_json::to_vec(&[payload])?;

    let agent = ureq::Agent::new_with_defaults();
    let _ = agent
        .post(phichain_telemetry::TELEMETRY_URL)
        .header("Content-Type", "application/json")
        .send(&*batch);

    let _ = std::fs::remove_file(&file);
    Ok(())
}
