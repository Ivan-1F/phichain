use bevy::{prelude::*, render::view::RenderLayers};

use crate::layer::GAME_LAYER;

#[derive(Component)]
pub struct Line;

#[derive(Component, Debug)]
pub struct LinePosition(pub Vec2);

#[derive(Component, Debug)]
pub struct LineRotation(pub f32);

#[derive(Component, Debug)]
pub struct LineOpacity(pub f32);

#[derive(Bundle)]
pub struct LineBundle {
    sprite: SpriteBundle,
    line: Line,
    position: LinePosition,
    rotation: LineRotation,
    opacity: LineOpacity,
    render_layers: RenderLayers,
}

impl LineBundle {
    pub fn new() -> Self {
        Self {
            sprite: SpriteBundle::default(),
            line: Line,
            position: LinePosition(Vec2 { x: 0.0, y: 0.0 }),
            rotation: LineRotation(0.0),
            opacity: LineOpacity(0.0),
            render_layers: RenderLayers::layer(GAME_LAYER),
        }
    }
}
