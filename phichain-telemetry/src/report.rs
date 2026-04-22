//! Fire-and-forget telemetry reporting for CLIs.
//!
//! [`Reporter::track`] writes a payload to a temp file and spawns the current
//! executable with `telemetry flush <path>` so the HTTP POST happens outside
//! the reporter's main process. Each phichain application must therefore
//! route `argv[1] == "telemetry"` to [`flush`] before its normal startup.

use crate::payload::{PhichainMeta, TelemetryPayload};
use serde_json::Value;
use std::fs::File;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const TELEMETRY_FILE_PREFIX: &str = "phichain-telemetry-";

/// Identifies the reporting application and is baked into every payload.
pub struct Reporter {
    name: &'static str,
    version: &'static str,
    debug: bool,
}

impl Reporter {
    /// `debug` should be `cfg!(debug_assertions)` from the caller's crate
    pub fn new(name: &'static str, version: &'static str, debug: bool) -> Self {
        Self {
            name,
            version,
            debug,
        }
    }

    /// Serialize a payload and hand it off to a flush subprocess.
    ///
    /// With `PHICHAIN_TELEMETRY_DEBUG` set, prints to stderr and returns
    /// instead of spawning anything.
    pub fn track(&self, event_type: &str, metadata: Value) -> std::io::Result<()> {
        let payload = TelemetryPayload::builder()
            .reporter(self.name)
            .event_type(event_type)
            .maybe_device_id(crate::device::get_device_id())
            .phichain(PhichainMeta::new(self.version, self.debug))
            .metadata(metadata)
            .build();

        if crate::env::telemetry_debug() {
            eprintln!("[telemetry] {}", serde_json::to_string_pretty(&payload)?);
            return Ok(());
        }

        let pid = std::process::id();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let path = std::env::temp_dir()
            .join(format!("{TELEMETRY_FILE_PREFIX}{pid}-{timestamp}.json"));
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
}

/// Entry point for the `<app> telemetry flush <path>` subcommand.
///
/// Validates that `file` sits in the system temp directory with the
/// expected prefix, POSTs its contents to the telemetry endpoint, then
/// removes the file. Errors are swallowed by the caller by convention so
/// an unreachable server never breaks the user's flow.
pub fn flush(file: PathBuf) -> std::io::Result<()> {
    let file = file.canonicalize()?;
    let temp_dir = std::env::temp_dir().canonicalize()?;
    if !file.starts_with(&temp_dir) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "telemetry file must be in temp directory",
        ));
    }

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

    let content = std::fs::read(&file)?;
    let payload: Value = serde_json::from_slice(&content).map_err(|_| {
        let _ = std::fs::remove_file(&file);
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "telemetry file contains invalid JSON",
        )
    })?;

    // Wrap in array to match the batch format the endpoint expects.
    let batch = serde_json::to_vec(&[payload])?;

    let agent = ureq::Agent::new_with_defaults();
    let _ = agent
        .post(crate::TELEMETRY_URL)
        .header("Content-Type", "application/json")
        .send(&*batch);

    let _ = std::fs::remove_file(&file);
    Ok(())
}
