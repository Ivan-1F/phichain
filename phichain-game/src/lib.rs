pub mod constants;
pub mod core;
pub mod highlight;
pub mod scale;

use crate::core::CoreGamePlugin;
use crate::highlight::HighlightPlugin;
use crate::scale::ScalePlugin;
use bevy::prelude::*;

/// The viewport for the game
#[derive(Debug, Clone, Resource)]
pub struct GameViewport(pub Rect);

/// Resource to control the chart time in seconds
#[derive(Debug, Clone, Resource)]
pub struct ChartTime(pub f32);

#[derive(Debug, Clone, Resource)]
pub struct GameConfig {
    note_scale: f32,
    fc_ap_indicator: bool,
    multi_highlight: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            note_scale: 1.0,
            fc_ap_indicator: true,
            multi_highlight: true,
        }
    }
}

/// System set for all systems related to game core
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameSet;

/// The Phigros Game Core Plugin
///
/// This plugin is responsible for:
///
/// - Updating translations for entities with [`Line`]s and [`Note`]s
/// - If [`GameConfig::multi_highlight`] is true, attach [`Highlighted`] for all notes with multi highlight
///
/// [`Line`]: phichain_chart::line::Line
/// [`Note`]: phichain_chart::note::Note
/// [`Highlighted`]: highlight::Highlighted
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameViewport(Rect::from_corners(Vec2::ZERO, Vec2::ZERO)))
            .insert_resource(ChartTime(0.0))
            .insert_resource(GameConfig::default())
            .add_plugins(HighlightPlugin)
            .add_plugins(ScalePlugin)
            .add_plugins(CoreGamePlugin);
    }
}
