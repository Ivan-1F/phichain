//! Environment detection utilities for telemetry.
//!
//! Provides functions to check telemetry opt-out, debug mode, CI environments,
//! and container runtimes.

use std::path::Path;

/// Checks whether a string value is truthy.
///
/// Recognizes `"true"`, `"yes"`, and `"1"` (case-insensitive).
fn is_truthy_value(value: &str) -> bool {
    matches!(value.to_lowercase().as_str(), "true" | "yes" | "1")
}

/// Checks whether an environment variable is set to a truthy value.
///
/// Returns `false` if the variable is unset or set to a non-truthy value.
fn env_var_is_truthy(name: &str) -> bool {
    std::env::var(name)
        .map(|v| is_truthy_value(&v))
        .unwrap_or(false)
}

/// Returns `true` if telemetry has been disabled via environment variables.
///
/// Checks both `PHICHAIN_TELEMETRY_DISABLED` and the standard
/// [`DO_NOT_TRACK`](https://consoledonottrack.com/) convention.
pub fn telemetry_disabled() -> bool {
    env_var_is_truthy("PHICHAIN_TELEMETRY_DISABLED") || env_var_is_truthy("DO_NOT_TRACK")
}

/// Returns `true` if telemetry debug mode is enabled.
///
/// When enabled, telemetry payloads are printed to stderr instead of
/// being sent to the remote endpoint. Controlled by `PHICHAIN_TELEMETRY_DEBUG`.
pub fn telemetry_debug() -> bool {
    env_var_is_truthy("PHICHAIN_TELEMETRY_DEBUG")
}

/// Returns `true` if running in a CI environment.
///
/// Checks for the presence of the `CI` environment variable, which is set
/// by most CI providers (GitHub Actions, GitLab CI, Travis CI, etc.).
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok()
}

/// Detects the container runtime, if any.
///
/// Checks for Kubernetes, Docker, and Podman via environment variables
/// and well-known file paths. Returns `None` when not running in a container.
pub fn container_environment() -> Option<&'static str> {
    if std::env::var("KUBERNETES_SERVICE_HOST").is_ok() {
        return Some("kubernetes");
    }

    if Path::new("/.dockerenv").exists() || Path::new("/run/.dockerenv").exists() {
        return Some("docker");
    }

    // The lowercase `container` env var is set by systemd-nspawn, Podman, and
    // some Docker configurations to indicate the container runtime.
    if let Ok(container) = std::env::var("container") {
        match container.to_lowercase().as_str() {
            "docker" => return Some("docker"),
            "podman" => return Some("podman"),
            _ => {}
        }
    }

    if Path::new("/run/.containerenv").exists() {
        return Some("podman");
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_truthy_value_recognizes_true_values() {
        for value in ["true", "True", "TRUE", "yes", "Yes", "YES", "1"] {
            assert!(is_truthy_value(value), "expected truthy for {value}");
        }
    }

    #[test]
    fn is_truthy_value_rejects_false_values() {
        for value in ["false", "no", "0", "", "anything", "2"] {
            assert!(!is_truthy_value(value), "expected falsy for {value}");
        }
    }
}
