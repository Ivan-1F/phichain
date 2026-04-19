use std::collections::BTreeMap;

use bevy::prelude::Resource;
use serde::Deserialize;

/// A string that may be localized.
///
/// In `meta.toml`, authors can write either a plain string or a table keyed by locale:
/// ```toml
/// # plain
/// name = "My Pack"
///
/// # localized
/// [name]
/// en_us = "My Pack"
/// zh_cn = "我的资源包"
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Localized {
    Single(String),
    Multi(BTreeMap<String, String>),
}

impl Localized {
    /// Resolve this field for `locale`. Falls back to `en_us` → `en` → any value → "".
    pub fn get(&self, locale: &str) -> &str {
        match self {
            Self::Single(s) => s,
            Self::Multi(map) => map
                .get(locale)
                .or_else(|| map.get("en_us"))
                .or_else(|| map.get("en"))
                .or_else(|| map.values().next())
                .map(String::as_str)
                .unwrap_or(""),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Single(s) => s.is_empty(),
            Self::Multi(map) => map.values().all(String::is_empty),
        }
    }
}

impl Default for Localized {
    fn default() -> Self {
        Self::Single(String::new())
    }
}

/// Resource pack metadata parsed from `meta.toml`.
#[derive(Debug, Clone, Resource, Deserialize)]
#[serde(default)]
pub struct RespackMeta {
    pub name: Localized,
    pub author: String,
    pub description: Localized,
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
            name: Localized::Single("Phichain Default".to_owned()),
            author: "Phichain".to_owned(),
            description: Localized::default(),
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
