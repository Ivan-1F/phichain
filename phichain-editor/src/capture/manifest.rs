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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Event>,
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
    Video {
        src: String,
        poster: String,
        width: u32,
        height: u32,
        scale: f32,
        /// Seconds
        duration: f64,
    },
}

/// Something that happens at `t` seconds into a video, for the documentation to show alongside
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    /// The numbered step of the instructions that starts here
    Step {
        t: f64,
        step: u32,
    },
    Key {
        t: f64,
        keys: String,
    },
    Click {
        t: f64,
        at: [f32; 2],
    },
    Highlight {
        t: f64,
        anchor: String,
        rect: Rect,
        until: f64,
    },
}
