use bevy::prelude::Resource;
use serde::Deserialize;

/// Resource pack metadata parsed from `meta.toml`.
///
/// This is phichain's native format. Phira compatibility is handled by a separate
/// adapter that converts Phira's `info.yml` into this representation.
#[derive(Debug, Clone, Resource, Deserialize)]
#[serde(default)]
pub struct RespackMeta {
    pub name: String,
    pub author: String,
    pub description: String,
    /// `[tail, head]` pixel heights for splitting `hold.png`.
    pub hold_atlas: [u32; 2],
    /// `[tail, head]` pixel heights for splitting `hold.highlight.png`.
    pub hold_highlight_atlas: [u32; 2],
    /// `[cols, rows]` grid dimensions for `hit.png` animation frames.
    pub hit_grid: [u32; 2],
    pub hide_particles: bool,
    pub hold_repeat: bool,
}

impl Default for RespackMeta {
    fn default() -> Self {
        Self {
            name: "Phichain Default".to_owned(),
            author: "Phichain".to_owned(),
            description: String::new(),
            hold_atlas: [50, 50],
            hold_highlight_atlas: [0, 110],
            hit_grid: [1, 30],
            hide_particles: false,
            hold_repeat: false,
        }
    }
}
