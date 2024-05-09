use bevy::{prelude::*, render::view::RenderLayers};

use crate::layer::GAME_LAYER;
use super::beat::Beat;

#[derive(Debug, Clone, Copy)]
pub enum NoteKind {
    Tap,
    Drag,
    Hold { hold_beat: Beat },
    Flick,
}

#[derive(Component, Debug)]
pub struct Note {
    pub kind: NoteKind,
    pub above: bool,
    pub beat: Beat,
    pub x: f32,
}

impl Note {
    pub fn new(kind: NoteKind, above: bool, beat: Beat, x: f32) -> Self {
        return Self {
            kind,
            above,
            beat,
            x,
        };
    }
}

#[derive(Component)]
pub struct TimelineNote(pub Entity);

#[derive(Bundle)]
pub struct NoteBundle {
    sprite: SpriteBundle,
    note: Note,
    render_layers: RenderLayers,
}

impl NoteBundle {
    pub fn new(note: Note) -> Self {
        Self {
            sprite: SpriteBundle::default(),
            note,
            render_layers: RenderLayers::layer(GAME_LAYER),
        }
    }
}
