use crate::beat::Beat;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteKind {
    Tap,
    Drag,
    Hold { hold_beat: Beat },
    Flick,
}

impl fmt::Debug for NoteKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NoteKind::Tap => write!(f, "Tap"),
            NoteKind::Drag => write!(f, "Drag"),
            NoteKind::Hold { hold_beat } => write!(f, "Hold({:?})", hold_beat),
            NoteKind::Flick => write!(f, "Flick"),
        }
    }
}

impl NoteKind {
    pub fn is_tap(&self) -> bool {
        matches!(self, NoteKind::Tap)
    }
    pub fn is_drag(&self) -> bool {
        matches!(self, NoteKind::Drag)
    }
    pub fn is_hold(&self) -> bool {
        matches!(self, NoteKind::Hold { .. })
    }
    pub fn is_flick(&self) -> bool {
        matches!(self, NoteKind::Flick)
    }
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct Note {
    pub kind: NoteKind,
    pub above: bool,
    pub beat: Beat,
    pub x: f32,
    pub speed: f32,
}

impl fmt::Debug for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(format!("{:?}", &self.kind).as_str())
            .field("above", &self.above)
            .field("beat", &self.beat)
            .field("x", &self.x)
            .field("speed", &self.speed)
            .finish()
    }
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

    /// Get the hold beat of this [`Note`] if possible
    ///
    /// Returns [`Some`] wrapping the inner `hold_beat` when self if a [`Hold`](NoteKind::Hold)
    ///
    /// Otherwise, return [`None`]
    pub fn hold_beat(&self) -> Option<&Beat> {
        match self.kind {
            NoteKind::Hold { ref hold_beat } => Some(hold_beat),
            _ => None,
        }
    }

    /// Get the mutable reference of the hold beat of this [`Note`] if possible
    ///
    /// Returns [`Some`] wrapping the mutable reference of the inner `hold_beat` when self if a [`Hold`](NoteKind::Hold)
    ///
    /// Otherwise, return [`None`]
    pub fn hold_beat_mut(&mut self) -> Option<&mut Beat> {
        match self.kind {
            NoteKind::Hold { ref mut hold_beat } => Some(hold_beat),
            _ => None,
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
        if let NoteKind::Hold { ref mut hold_beat } = self.kind {
            *hold_beat = end_beat - self.beat
        }
    }
}

#[cfg(feature = "bevy")]
#[derive(bevy::prelude::Bundle)]
pub struct NoteBundle {
    sprite: bevy::prelude::Sprite,
    note: Note,
}

#[cfg(feature = "bevy")]
impl NoteBundle {
    pub fn new(note: Note) -> Self {
        Self {
            sprite: bevy::prelude::Sprite::default(),
            note,
        }
    }
}
