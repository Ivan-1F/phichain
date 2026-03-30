//! Telemetry payload construction.
//!
//! Provides a [`TelemetryPayload`] builder for assembling telemetry events
//! with common fields auto-populated and optional app-specific extensions.

use bon::Builder;
use serde::Serialize;
use serde_json::Value;

use crate::env;

/// Operating system and architecture information.
#[derive(Debug, Clone, Serialize)]
pub struct SystemInfo {
    pub arch: &'static str,
    pub os: &'static str,
    pub name: String,
    pub version: String,
}

impl SystemInfo {
    /// Collects system information from the current environment.
    pub fn collect() -> Self {
        let info = os_info::get();
        Self {
            arch: std::env::consts::ARCH,
            os: std::env::consts::OS,
            name: info.os_type().to_string(),
            version: info.version().to_string(),
        }
    }
}

/// Runtime environment metadata.
#[derive(Debug, Clone, Serialize)]
pub struct EnvironmentInfo {
    pub container: &'static str,
    pub ci: bool,
    pub test: bool,
}

impl EnvironmentInfo {
    /// Collects environment information.
    ///
    /// `is_test` should be passed as `cfg!(test)` by the caller, since
    /// the `cfg!` macro is evaluated in the caller's compilation context,
    /// not the library's.
    pub fn collect(is_test: bool) -> Self {
        Self {
            container: env::container_environment().unwrap_or("none"),
            ci: env::is_ci(),
            test: is_test,
        }
    }
}

/// Phichain application metadata.
#[derive(Debug, Clone, Builder, Serialize)]
pub struct PhichainMeta {
    /// Whether this is a beta release. Defaults to checking if `version`
    /// contains `"beta"`, but can be overridden explicitly.
    pub beta: bool,
    pub version: String,
    pub debug: bool,
}

impl PhichainMeta {
    /// Creates metadata from the caller's package version and debug status.
    ///
    /// The `beta` flag is derived automatically by checking whether `version`
    /// contains the substring `"beta"`.
    ///
    /// `is_debug` should be passed as `cfg!(debug_assertions)` by the caller.
    pub fn new(version: &str, is_debug: bool) -> Self {
        Self {
            beta: version.contains("beta"),
            version: version.to_owned(),
            debug: is_debug,
        }
    }
}

/// A telemetry event payload with common fields auto-populated.
///
/// Use [`TelemetryPayload::builder()`] to construct:
///
/// ```ignore
/// let payload = TelemetryPayload::builder()
///     .reporter("phichain-converter")
///     .event_type("phichain.converter.convert")
///     .phichain(PhichainMeta::new(env!("CARGO_PKG_VERSION"), cfg!(debug_assertions)))
///     .metadata(json!({"key": "value"}))
///     .build();
/// ```
#[derive(Debug, Clone, Builder, Serialize)]
#[builder(on(String, into))]
pub struct TelemetryPayload {
    /// Additional app-specific fields merged into the top-level JSON object.
    /// Each entry becomes a top-level key in the serialized output.
    #[serde(flatten)]
    #[builder(field)]
    pub extra: std::collections::HashMap<String, Value>,

    /// RFC 3339 timestamp. Auto-populated if not set.
    #[builder(default = default_timestamp())]
    pub timestamp: String,

    /// The reporting application name (e.g., `"phichain-editor"`).
    pub reporter: String,

    /// Stable anonymous device identifier, or `null` when unavailable.
    pub device_id: Option<String>,

    /// The event type (e.g., `"phichain.editor.started"`).
    #[serde(rename = "type")]
    pub event_type: String,

    /// OS and architecture info. Auto-populated if not set.
    #[builder(default = SystemInfo::collect())]
    pub system: SystemInfo,

    /// Container/CI/test environment info. Auto-populated if not set.
    #[builder(default = EnvironmentInfo::collect(false))]
    pub environment: EnvironmentInfo,

    /// Application version metadata.
    pub phichain: PhichainMeta,

    /// Application-specific event data.
    pub metadata: Value,
}

impl<S: telemetry_payload_builder::State> TelemetryPayloadBuilder<S> {
    /// Inserts an additional top-level field into the payload.
    ///
    /// Can be called multiple times to add more fields:
    /// ```ignore
    /// .extra("session_id", json!(uuid))
    /// .extra("hardware", json!({...}))
    /// ```
    pub fn extra(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        self.extra
            .insert(key.into(), serde_json::to_value(value).unwrap());
        self
    }
}

fn default_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_info_has_non_empty_fields() {
        let info = SystemInfo::collect();
        assert!(!info.arch.is_empty());
        assert!(!info.os.is_empty());
        assert!(!info.name.is_empty());
        assert!(!info.version.is_empty());
    }

    #[test]
    fn phichain_meta_detects_beta() {
        let meta = PhichainMeta::new("1.0.0-beta.5", false);
        assert!(meta.beta);

        let meta = PhichainMeta::new("1.0.0", false);
        assert!(!meta.beta);
    }

    #[test]
    fn phichain_meta_preserves_debug() {
        let meta = PhichainMeta::new("1.0.0", true);
        assert!(meta.debug);

        let meta = PhichainMeta::new("1.0.0", false);
        assert!(!meta.debug);
    }

    #[test]
    fn environment_info_collect() {
        let info = EnvironmentInfo::collect(false);
        assert!(!info.test);
    }

    #[test]
    fn builder_produces_valid_payload() {
        let payload = TelemetryPayload::builder()
            .reporter("test-reporter")
            .event_type("test.event")
            .phichain(PhichainMeta::new("1.0.0", false))
            .metadata(serde_json::json!({"key": "value"}))
            .build();

        assert_eq!(payload.reporter, "test-reporter");
        assert_eq!(payload.event_type, "test.event");
        assert!(payload.device_id.is_none());
        assert!(!payload.timestamp.is_empty());
        assert!(payload.extra.is_empty());
    }

    #[test]
    fn builder_with_extra_fields() {
        let payload = TelemetryPayload::builder()
            .reporter("test")
            .event_type("test")
            .phichain(PhichainMeta::new("1.0.0", false))
            .metadata(serde_json::json!({}))
            .extra("session_id", serde_json::json!("session-123"))
            .extra("uptime", serde_json::json!(42.5))
            .build();

        let json = serde_json::to_value(&payload).unwrap();
        let obj = json.as_object().unwrap();
        assert_eq!(obj["session_id"], "session-123");
        assert_eq!(obj["uptime"], 42.5);
    }

    #[test]
    fn extra_fields_are_flattened() {
        let payload = TelemetryPayload::builder()
            .reporter("test")
            .event_type("test")
            .phichain(PhichainMeta::new("1.0.0", false))
            .metadata(serde_json::json!({}))
            .build();

        let json = serde_json::to_value(&payload).unwrap();
        let obj = json.as_object().unwrap();

        // No extra fields in output
        assert!(!obj.contains_key("extra"));

        // Required fields should appear
        assert!(obj.contains_key("reporter"));
        assert!(obj.contains_key("type"));
        assert!(obj.contains_key("device_id"));
    }
}
