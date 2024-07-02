use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

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

impl PartialOrd for Note {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.beat.partial_cmp(&other.beat)
    }
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

    /// Get the end beat of this [`Note`]
    ///
    /// If `self.kind` is not [`NoteKind::Hold`], this returns the beat of this [`Note`]
    pub fn end_beat(&self) -> Beat {
        match self.kind {
            NoteKind::Hold { hold_beat } => self.beat + hold_beat,
            _ => self.beat,
        }
    }
    
    /// Set the end beat of this [`Note`]
    /// 
    /// This only has effect when `self.kind` is [`NoteKind::Hold`]
    pub fn set_end_beat(&mut self, end_beat: Beat) {
        match self.kind {
            NoteKind::Hold { ref mut hold_beat } => *hold_beat = end_beat - self.beat,
            _ => {},
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
