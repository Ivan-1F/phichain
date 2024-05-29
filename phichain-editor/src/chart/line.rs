use bevy::prelude::*;

#[derive(Component)]
pub struct Line;

#[derive(Component, Debug, Default)]
pub struct LinePosition(pub Vec2);

#[derive(Component, Debug, Default)]
pub struct LineRotation(pub f32);

#[derive(Component, Debug, Default)]
pub struct LineOpacity(pub f32);

/// This will not affect line entity, it is only used to show realtime speed of lines in [crate::tab::line_list]
#[derive(Component, Debug, Default)]
pub struct LineSpeed(pub f32);

#[derive(Bundle)]
pub struct LineBundle {
    sprite: SpriteBundle,
    line: Line,
    position: LinePosition,
    rotation: LineRotation,
    opacity: LineOpacity,
    speed: LineSpeed,
}

impl LineBundle {
    pub fn new() -> Self {
        Self {
            sprite: SpriteBundle::default(),
            line: Line,
            position: Default::default(),
            rotation: Default::default(),
            opacity: Default::default(),
            speed: Default::default(),
        }
    }
}
