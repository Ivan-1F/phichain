use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::meta::ResPackMeta;

use super::{load_audio, load_image, LoadedAudio, LoadedImages, LoadedResPack};

/// Phira's `info.yml` schema. Unsupported fields (`hitFxDuration`,
/// `colorPerfect`, etc.) are silently ignored.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct PhiraInfo {
    name: String,
    author: String,
    description: String,
    #[serde(rename = "holdAtlas")]
    hold_atlas: [u32; 2],
    #[serde(rename = "holdAtlasMH")]
    hold_atlas_mh: [u32; 2],
    #[serde(rename = "hitFx")]
    hit_fx: [u32; 2],
    #[serde(rename = "hideParticles")]
    hide_particles: bool,
    #[serde(rename = "holdRepeat")]
    hold_repeat: bool,
}

impl Default for PhiraInfo {
    fn default() -> Self {
        let fallback = ResPackMeta::default();
        Self {
            name: fallback.name,
            author: fallback.author,
            description: fallback.description,
            hold_atlas: fallback.hold_atlas,
            hold_atlas_mh: fallback.hold_highlight_atlas,
            hit_fx: fallback.hit_grid,
            hide_particles: fallback.hide_particles,
            hold_repeat: fallback.hold_repeat,
        }
    }
}

impl From<PhiraInfo> for ResPackMeta {
    fn from(info: PhiraInfo) -> Self {
        Self {
            name: info.name,
            author: info.author,
            description: info.description,
            hold_atlas: info.hold_atlas,
            hold_highlight_atlas: info.hold_atlas_mh,
            hit_grid: info.hit_fx,
            hide_particles: info.hide_particles,
            hold_repeat: info.hold_repeat,
        }
    }
}

/// Load a Phira-format resource pack from a filesystem directory.
pub fn load(dir: &Path) -> Result<LoadedResPack> {
    let image = |name| load_image(dir, name);
    let audio = |name| load_audio(dir, name);

    Ok(LoadedResPack {
        meta: load_info(dir)?.into(),
        images: LoadedImages {
            tap: image("click.png")?,
            tap_highlight: image("click_mh.png")?,
            drag: image("drag.png")?,
            drag_highlight: image("drag_mh.png")?,
            flick: image("flick.png")?,
            flick_highlight: image("flick_mh.png")?,
            hold: image("hold.png")?,
            hold_highlight: image("hold_mh.png")?,
            hit: image("hit_fx.png")?,
            line: image("line.png")?,
        },
        audio: LoadedAudio {
            tap: audio("click.ogg")?,
            drag: audio("drag.ogg")?,
            flick: audio("flick.ogg")?,
        },
    })
}

fn load_info(dir: &Path) -> Result<PhiraInfo> {
    let path = dir.join("info.yml");
    let data = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    serde_yaml::from_str(&data).with_context(|| format!("failed to parse {}", path.display()))
}
