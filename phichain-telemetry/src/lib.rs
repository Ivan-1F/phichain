//! Shared telemetry utilities for phichain applications.
//!
//! This crate provides building blocks for anonymous telemetry collection
//! without depending on any application-specific framework (Bevy, etc.).
//!
//! # Modules
//!
//! - [`adapter`] - GPU adapter fingerprint (opt-in via `wgpu` feature for the `From<&wgpu_types::AdapterInfo>` impl)
//! - [`device`] - Stable, anonymous device identification
//! - [`env`] - Environment detection (opt-out, debug mode, CI, containers)
//! - [`hardware`] - CPU / memory fingerprint
//! - [`payload`] - Common payload construction

pub mod adapter;
pub mod device;
pub mod env;
pub mod hardware;
pub mod payload;
pub mod report;

pub use report::{Reporter, flush, handle_subcommand, send};

/// The telemetry reporting endpoint.
pub const TELEMETRY_URL: &str = "https://telemetry.phichain.rs/report";
