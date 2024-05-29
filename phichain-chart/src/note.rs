use serde::{Deserialize, Serialize};

use crate::beat::Beat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteKind {
    Tap,
    Drag,
    Hold { hold_beat: Beat },
    Flick,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct Note {
    pub kind: NoteKind,
    pub above: bool,
    pub beat: Beat,
    pub x: f32,
    pub speed: f32,
}

impl Note {
    pub fn new(kind: NoteKind, above: bool, beat: Beat, x: f32, speed: f32) -> Self {
        Self {
            kind,
            above,
            beat,
            x,
            speed,
        }
    }
}

#[cfg(feature = "bevy")]
#[derive(bevy::prelude::Bundle)]
pub struct NoteBundle {
    sprite: bevy::prelude::SpriteBundle,
    note: Note,
}

#[cfg(feature = "bevy")]
impl NoteBundle {
    pub fn new(note: Note) -> Self {
        Self {
            sprite: bevy::prelude::SpriteBundle::default(),
            note,
        }
    }
}
