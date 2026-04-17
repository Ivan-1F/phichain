mod native;
mod phira;
mod source;

use std::path::Path;

use anyhow::{Context, Result};
use image::DynamicImage;

use crate::meta::ResPackMeta;

use source::PackSource;

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
pub fn load_respack_from_dir(dir: &Path) -> Result<LoadedResPack> {
    load(PackSource::open_dir(dir)?)
}

/// Load a resource pack from a ZIP archive on disk.
pub fn load_respack_from_zip(path: &Path) -> Result<LoadedResPack> {
    load(PackSource::open_zip(path)?)
}

fn load(mut source: PackSource) -> Result<LoadedResPack> {
    if source.exists("info.yml") {
        phira::load(&mut source)
    } else {
        native::load(&mut source)
    }
}

fn load_image(source: &mut PackSource, name: &str) -> Result<DynamicImage> {
    let bytes = source.read(name)?;
    image::load_from_memory(&bytes).with_context(|| format!("failed to decode image: {name}"))
}
