use std::path::Path;

use anyhow::{Context, Result};

use crate::meta::ResPackMeta;

use super::{load_audio, load_image, LoadedAudio, LoadedImages, LoadedResPack};

/// Load a phichain-native resource pack from a filesystem directory.
///
/// `meta.toml` is optional; default metadata is used when it is missing.
pub fn load(dir: &Path) -> Result<LoadedResPack> {
    let image = |name| load_image(dir, name);
    let audio = |name| load_audio(dir, name);

    Ok(LoadedResPack {
        meta: load_meta(dir)?,
        images: LoadedImages {
            tap: image("tap.png")?,
            tap_highlight: image("tap.highlight.png")?,
            drag: image("drag.png")?,
            drag_highlight: image("drag.highlight.png")?,
            flick: image("flick.png")?,
            flick_highlight: image("flick.highlight.png")?,
            hold: image("hold.png")?,
            hold_highlight: image("hold.highlight.png")?,
            hit: image("hit.png")?,
            line: image("line.png")?,
        },
        audio: LoadedAudio {
            tap: audio("tap.ogg")?,
            drag: audio("drag.ogg")?,
            flick: audio("flick.ogg")?,
        },
    })
}

fn load_meta(dir: &Path) -> Result<ResPackMeta> {
    let path = dir.join("meta.toml");
    match std::fs::read_to_string(&path) {
        Ok(data) => toml::from_str(&data)
            .with_context(|| format!("failed to parse {}", path.display())),
        // Missing meta.toml is acceptable — fall back to defaults.
        Err(_) => Ok(ResPackMeta::default()),
    }
}
