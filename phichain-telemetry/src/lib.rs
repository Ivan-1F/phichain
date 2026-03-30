//! Shared telemetry utilities for phichain applications.
//!
//! This crate provides building blocks for anonymous telemetry collection
//! without depending on any application-specific framework (Bevy, etc.).
//!
//! # Modules
//!
//! - [`device`] - Stable, anonymous device identification
//! - [`env`] - Environment detection (opt-out, debug mode, CI, containers)
//! - [`payload`] - Common payload construction

pub mod device;
pub mod env;
pub mod payload;

/// The telemetry reporting endpoint.
pub const TELEMETRY_URL: &str = "https://telemetry.phichain.rs/report";
