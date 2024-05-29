use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::beat::Beat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteKind {
    Tap,
    Drag,
    Hold { hold_beat: Beat },
    Flick,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

#[derive(Bundle)]
pub struct NoteBundle {
    sprite: SpriteBundle,
    note: Note,
}

impl NoteBundle {
    pub fn new(note: Note) -> Self {
        Self {
            sprite: SpriteBundle::default(),
            note,
        }
    }
}
