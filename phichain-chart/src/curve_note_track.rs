use crate::easing::Easing;
use crate::note::NoteKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveNoteTrackOptions {
    pub kind: NoteKind,
    pub density: u32,
    pub curve: Easing,
}

impl Default for CurveNoteTrackOptions {
    fn default() -> Self {
        Self {
            kind: NoteKind::Drag,
            density: 16,
            curve: Easing::Linear,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveNoteTrack {
    pub from: usize,
    pub to: usize,

    #[serde(flatten)]
    pub options: CurveNoteTrackOptions,
}
