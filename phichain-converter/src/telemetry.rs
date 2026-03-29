use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn get_device_id() -> String {
    let machine_id = machine_uid::get().unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());
    let mut hasher = Sha256::new();
    hasher.update(machine_id.as_bytes());
    let hash_result = hasher.finalize();
    hash_result.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn disabled() -> bool {
    std::env::var("PHICHAIN_TELEMETRY_DISABLED")
        .map(|v| matches!(v.to_lowercase().as_str(), "true" | "yes" | "1"))
        .unwrap_or(false)
        || std::env::var("DO_NOT_TRACK")
            .map(|v| matches!(v.to_lowercase().as_str(), "true" | "yes" | "1"))
            .unwrap_or(false)
}

pub fn track(event_type: &str, metadata: Value) -> Result<(), std::io::Error> {
    let info = os_info::get();
    let payload = serde_json::json!({
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        "reporter": "phichain-converter",
        "device_id": get_device_id(),
        "type": event_type,
        "system": {
            "arch": std::env::consts::ARCH,
            "os": std::env::consts::OS,
            "name": info.os_type().to_string(),
            "version": info.version().to_string(),
        },
        "phichain": {
            "version": env!("CARGO_PKG_VERSION"),
            "debug": cfg!(debug_assertions),
        },
        "metadata": metadata,
    });

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
    if serde_json::from_slice::<Value>(&content).is_err() {
        let _ = std::fs::remove_file(&file);
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "telemetry file contains invalid JSON",
        ));
    }

    // TODO: replace with actual HTTP POST
    eprintln!("[telemetry] {}", String::from_utf8_lossy(&content));

    let _ = std::fs::remove_file(&file);
    Ok(())
}
