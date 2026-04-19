use anyhow::{Context, Result};
use serde::Deserialize;

use crate::meta::{HitFxMeta, HoldMeta, RespackMeta};

use super::source::PackSource;
use super::{load_image, LoadedAudio, LoadedImages, LoadedRespack};

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
    #[serde(rename = "hitFxScale")]
    hit_fx_scale: f32,
    #[serde(rename = "hitFxDuration")]
    hit_fx_duration: f32,
}

impl Default for PhiraInfo {
    fn default() -> Self {
        let fallback = RespackMeta::default();
        Self {
            name: fallback.name,
            author: fallback.author,
            description: fallback.description,
            hold_atlas: fallback.hold.atlas,
            hold_atlas_mh: fallback.hold.highlight_atlas,
            hit_fx: fallback.hit_fx.grid,
            hit_fx_scale: fallback.hit_fx.scale,
            hit_fx_duration: fallback.hit_fx.duration,
        }
    }
}

impl From<PhiraInfo> for RespackMeta {
    fn from(info: PhiraInfo) -> Self {
        Self {
            name: info.name,
            author: info.author,
            description: info.description,
            hold: HoldMeta {
                atlas: info.hold_atlas,
                highlight_atlas: info.hold_atlas_mh,
            },
            hit_fx: HitFxMeta {
                grid: info.hit_fx,
                scale: info.hit_fx_scale,
                duration: info.hit_fx_duration,
            },
        }
    }
}

/// Load a Phira-format resource pack.
pub fn load(source: &mut PackSource) -> Result<LoadedRespack> {
    Ok(LoadedRespack {
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
