use anyhow::{Context, Result};
use serde::Deserialize;

use crate::meta::ResPackMeta;

use super::source::PackSource;
use super::{load_image, LoadedAudio, LoadedImages, LoadedResPack};

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

/// Load a Phira-format resource pack.
pub fn load(source: &mut PackSource) -> Result<LoadedResPack> {
    Ok(LoadedResPack {
        meta: load_info(source)?.into(),
        images: LoadedImages {
            tap: load_image(source, "click.png")?,
            tap_highlight: load_image(source, "click_mh.png")?,
            drag: load_image(source, "drag.png")?,
            drag_highlight: load_image(source, "drag_mh.png")?,
            flick: load_image(source, "flick.png")?,
            flick_highlight: load_image(source, "flick_mh.png")?,
            hold: load_image(source, "hold.png")?,
            hold_highlight: load_image(source, "hold_mh.png")?,
            hit: load_image(source, "hit_fx.png")?,
            line: load_image(source, "line.png")?,
        },
        audio: LoadedAudio {
            tap: source.read("click.ogg")?,
            drag: source.read("drag.ogg")?,
            flick: source.read("flick.ogg")?,
        },
    })
}

fn load_info(source: &mut PackSource) -> Result<PhiraInfo> {
    let bytes = source.read("info.yml")?;
    let text = std::str::from_utf8(&bytes).context("info.yml is not valid UTF-8")?;
    serde_yaml::from_str(text).context("failed to parse info.yml")
}
