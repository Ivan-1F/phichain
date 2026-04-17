mod native;
mod phira;

use std::path::Path;

use anyhow::{bail, Context, Result};
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

/// Load a resource pack from a filesystem directory.
///
/// The format is auto-detected:
/// - If `info.yml` exists, the directory is treated as a Phira resource pack.
/// - Otherwise, the directory is treated as a phichain-native resource pack
///   (`meta.toml` is optional; defaults are used when absent).
pub fn load_respack_from_dir(dir: &Path) -> Result<LoadedResPack> {
    if !dir.is_dir() {
        bail!("resource pack directory does not exist: {}", dir.display());
    }
    if dir.join("info.yml").exists() {
        phira::load(dir)
    } else {
        native::load(dir)
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
