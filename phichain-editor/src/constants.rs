use bevy::prelude::Color;
pub use phichain_chart::constants::*;

pub const INDICATOR_POSITION: f32 = 0.85;
pub const BASE_ZOOM: f32 = 400.0;

// TODO: make below constants customizable in editor settings
pub const ILLUSTRATION_BLUR: f32 = 160.0;
pub const ILLUSTRATION_ALPHA: f32 = 0.2;

// the color for perfect hit particles and lines
// #feffa9
pub const PERFECT_COLOR: Color = Color::rgb(254.0 / 255.0, 1.0, 169.0 / 255.0);
