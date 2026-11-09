use serde::Serialize;
use std::collections::BTreeMap;

/// `[x, y, width, height]` in media pixels
pub type Rect = [f32; 4];

/// Describes one captured media file for the documentation site
#[derive(Serialize)]
pub struct Manifest {
    pub version: u32,
    pub id: String,
    pub media: Media,
    pub anchors: BTreeMap<String, Rect>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Media {
    Image {
        src: String,
        width: u32,
        height: u32,
        /// Media pixels per CSS pixel when shown at the size it has in the editor
        scale: f32,
    },
}
