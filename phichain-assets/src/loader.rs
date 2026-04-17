use std::path::Path;

use anyhow::{Context, Result};
use image::DynamicImage;

use crate::meta::ResPackMeta;

/// A resource pack fully decoded in memory, ready to be applied to Bevy.
pub struct LoadedResPack {
    pub meta: ResPackMeta,
    pub images: LoadedImages,
    pub audio: LoadedAudio,
}

pub struct LoadedImages {
    pub tap: DynamicImage,
    pub tap_highlight: DynamicImage,
    pub drag: DynamicImage,
    pub drag_highlight: DynamicImage,
    pub flick: DynamicImage,
    pub flick_highlight: DynamicImage,
    pub hold: DynamicImage,
    pub hold_highlight: DynamicImage,
    pub hit: DynamicImage,
    pub line: DynamicImage,
}

pub struct LoadedAudio {
    pub tap: Vec<u8>,
    pub drag: Vec<u8>,
    pub flick: Vec<u8>,
}

/// Load a phichain-native resource pack from a filesystem directory.
///
/// The directory must contain all required image and audio files according to
/// the phichain native resource pack spec. `meta.toml` is optional; default
/// metadata is used when it is missing.
pub fn load_respack_from_dir(dir: &Path) -> Result<LoadedResPack> {
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

fn load_image(dir: &Path, name: &str) -> Result<DynamicImage> {
    let path = dir.join(name);
    image::open(&path).with_context(|| format!("failed to load image: {}", path.display()))
}

fn load_audio(dir: &Path, name: &str) -> Result<Vec<u8>> {
    let path = dir.join(name);
    std::fs::read(&path).with_context(|| format!("failed to load audio: {}", path.display()))
}
