use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub struct Line;

#[derive(Component, Debug, Default)]
pub struct LinePosition(pub Vec2);

#[derive(Component, Debug, Default)]
pub struct LineRotation(pub f32);

#[derive(Component, Debug, Default)]
pub struct LineOpacity(pub f32);

/// This will not affect line entity, it is only used to show realtime speed of lines in [phichain::tab::line_list]
#[derive(Component, Debug, Default)]
pub struct LineSpeed(pub f32);

#[derive(Bundle, Default)]
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
        Self::default()
    }
}
