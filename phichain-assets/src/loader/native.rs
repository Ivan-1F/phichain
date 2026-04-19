use anyhow::{Context, Result};

use crate::meta::RespackMeta;

use super::source::PackSource;
use super::{load_image, LoadedAudio, LoadedImages, LoadedRespack};

/// Load a phichain-native resource pack.
///
/// `meta.toml` is optional; default metadata is used when it is missing.
pub fn load(source: &mut PackSource) -> Result<LoadedRespack> {
    Ok(LoadedRespack {
        meta: load_meta(source)?,
        images: LoadedImages {
            tap: load_image(source, "tap.png")?,
            tap_highlight: load_image(source, "tap.highlight.png")?,
            drag: load_image(source, "drag.png")?,
            drag_highlight: load_image(source, "drag.highlight.png")?,
            flick: load_image(source, "flick.png")?,
            flick_highlight: load_image(source, "flick.highlight.png")?,
            hold: load_image(source, "hold.png")?,
            hold_highlight: load_image(source, "hold.highlight.png")?,
            hit: load_image(source, "hit.png")?,
            line: load_image(source, "line.png")?,
        },
        audio: LoadedAudio {
            tap: source.read("tap.ogg")?,
            drag: source.read("drag.ogg")?,
            flick: source.read("flick.ogg")?,
        },
    })
}

pub(super) fn load_meta(source: &mut PackSource) -> Result<RespackMeta> {
    if !source.exists("meta.toml") {
        return Ok(RespackMeta::default());
    }
    let bytes = source.read("meta.toml")?;
    let text = std::str::from_utf8(&bytes).context("meta.toml is not valid UTF-8")?;
    toml::from_str(text).context("failed to parse meta.toml")
}
