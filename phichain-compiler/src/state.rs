use nalgebra::{Isometry2, Rotation2, Vector2};
use phichain_chart::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};

/// Represents the state of a line
#[derive(Debug, Copy, Clone)]
pub struct LineState {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub opacity: f32,
    #[allow(dead_code)]
    pub speed: f32,
}

impl LineState {
    /// Returns if the line is visible in the viewport
    pub fn is_visible(&self) -> bool {
        if self.opacity <= 0.0 {
            false
        } else if self.x >= 0.0
            && self.x <= CANVAS_WIDTH
            && self.y >= 0.0
            && self.y <= CANVAS_HEIGHT
        {
            true
        } else {
            Isometry2::new(
                Vector2::new(self.x, self.y),
                Rotation2::new(self.rotation.to_radians()).angle(),
            );

            true
        }
    }
}
