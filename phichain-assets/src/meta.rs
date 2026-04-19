use bevy::prelude::Resource;
use serde::Deserialize;

/// Resource pack metadata parsed from `meta.toml`.
#[derive(Debug, Clone, Resource, Deserialize)]
#[serde(default)]
pub struct RespackMeta {
    pub name: String,
    pub author: String,
    pub description: String,
    pub hold: HoldMeta,
    pub hit_fx: HitFxMeta,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct HoldMeta {
    /// `[tail, head]` pixel heights for splitting `hold.png`.
    pub atlas: [u32; 2],
    /// `[tail, head]` pixel heights for splitting `hold.highlight.png`.
    pub highlight_atlas: [u32; 2],
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct HitFxMeta {
    /// `[cols, rows]` grid dimensions for `hit.png` animation frames.
    pub grid: [u32; 2],
    pub scale: f32,
    /// Total hit effect animation duration in seconds.
    pub duration: f32,
}

impl Default for RespackMeta {
    fn default() -> Self {
        Self {
            name: "Phichain Default".to_owned(),
            author: "Phichain".to_owned(),
            description: String::new(),
            hold: HoldMeta::default(),
            hit_fx: HitFxMeta::default(),
        }
    }
}

impl Default for HoldMeta {
    fn default() -> Self {
        Self {
            atlas: [50, 50],
            highlight_atlas: [0, 110],
        }
    }
}

impl Default for HitFxMeta {
    fn default() -> Self {
        Self {
            grid: [1, 30],
            scale: 1.0,
            duration: 0.5,
        }
    }
}
