mod native;
mod phira;
mod source;

use std::path::Path;

use anyhow::{Context, Result};
use image::DynamicImage;

use crate::meta::RespackMeta;

use source::PackSource;

/// A resource pack fully decoded in memory, ready to be applied to Bevy.
pub struct LoadedRespack {
    pub meta: RespackMeta,
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
pub fn load_respack_from_dir(dir: &Path) -> Result<LoadedRespack> {
    load(PackSource::open_dir(dir)?)
}

/// Load a resource pack from a ZIP archive on disk.
pub fn load_respack_from_zip(path: &Path) -> Result<LoadedRespack> {
    load(PackSource::open_zip(path)?)
}

/// Load a resource pack from any path
pub fn load_respack(path: &Path) -> Result<LoadedRespack> {
    if path.is_dir() {
        load_respack_from_dir(path)
    } else {
        load_respack_from_zip(path)
    }
}

/// Load only the metadata of a resource pack, without decoding images or audio.
///
/// Useful for listing available packs in UI without paying the cost of fully loading each one.
pub fn load_respack_meta(path: &Path) -> Result<RespackMeta> {
    let mut source = if path.is_dir() {
        PackSource::open_dir(path)?
    } else {
        PackSource::open_zip(path)?
    };
    if source.exists("info.yml") {
        phira::load_meta(&mut source)
    } else {
        native::load_meta(&mut source)
    }
}

/// Preview images (note textures) used by UI to show a thumbnail for each pack.
#[derive(Debug)]
pub struct LoadedRespackPreview {
    pub tap: DynamicImage,
    pub drag: DynamicImage,
    pub flick: DynamicImage,
    pub hold: DynamicImage,
}

/// Load the four note-type textures from a resource pack.
///
/// This is lighter than [`load_respack`] because it skips highlight variants, hit effects,
/// the judge line, and all audio. Intended for the settings pack-picker.
pub fn load_respack_preview(path: &Path) -> Result<LoadedRespackPreview> {
    let mut source = if path.is_dir() {
        PackSource::open_dir(path)?
    } else {
        PackSource::open_zip(path)?
    };
    // Phira packs use `click.png` instead of `tap.png`; everything else matches.
    let tap_name = if source.exists("info.yml") {
        "click.png"
    } else {
        "tap.png"
    };
    Ok(LoadedRespackPreview {
        tap: load_image(&mut source, tap_name)?,
        drag: load_image(&mut source, "drag.png")?,
        flick: load_image(&mut source, "flick.png")?,
        hold: load_image(&mut source, "hold.png")?,
    })
}

fn load(mut source: PackSource) -> Result<LoadedRespack> {
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

/// Like [`load_image`], but returns `None` if the file does not exist in the pack.
/// A decoding failure on an existing file is still surfaced as an error.
fn load_image_opt(source: &mut PackSource, name: &str) -> Result<Option<DynamicImage>> {
    if !source.exists(name) {
        return Ok(None);
    }
    load_image(source, name).map(Some)
}

/// Decode the built-in `line.png` embedded at compile time.
///
/// Used as a fallback when a resource pack does not ship its own `line.png`
/// (e.g. phira-format packs, which don't support a custom judge line texture).
pub(super) fn builtin_line() -> DynamicImage {
    const BYTES: &[u8] = include_bytes!("../../../assets/respack/line.png");
    image::load_from_memory(BYTES).expect("built-in line.png should decode")
}
