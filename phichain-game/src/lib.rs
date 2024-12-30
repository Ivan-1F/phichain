pub mod constants;
pub mod core;
pub mod curve_note_track;
pub mod highlight;
mod hit_effect;
pub mod illustration;
mod layer;
mod loader;
pub mod scale;
mod score;
mod ui;

pub use crate::loader::load_project;

use crate::core::CoreGamePlugin;
use crate::curve_note_track::CurveNoteTrackPlugin;
use crate::highlight::HighlightPlugin;
use crate::hit_effect::HitEffectPlugin;
use crate::illustration::IllustrationPlugin;
use crate::scale::ScalePlugin;
use crate::score::ScorePlugin;
use crate::ui::GameUiPlugin;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::ShapePlugin;

/// The viewport for the game
#[derive(Debug, Clone, Resource)]
pub struct GameViewport(pub Rect);

/// If the chart is paused
#[derive(Debug, Clone, Resource)]
pub struct Paused(pub bool);

/// Resource to control the chart time in seconds
#[derive(Debug, Clone, Resource)]
pub struct ChartTime(pub f32);

#[derive(Debug, Clone, Resource)]
pub struct GameConfig {
    pub note_scale: f32,
    pub fc_ap_indicator: bool,
    pub multi_highlight: bool,
    pub hide_hit_effect: bool,

    pub name: String,
    pub level: String,

    /// If enabled, hit effects will use [`ChartTime`] instead of [`Time`] for calculation
    ///
    /// This is useful in the renderer
    pub hit_effect_follow_game_time: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            note_scale: 1.0,
            fc_ap_indicator: true,
            multi_highlight: true,
            hide_hit_effect: false,

            name: Default::default(),
            level: Default::default(),

            hit_effect_follow_game_time: false,
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
/// - Hit effects (including animations and particles)
/// - Generating and managing [`CurveNote`]s based on [`CurveNoteTrack`]s
///
/// [`Line`]: phichain_chart::line::Line
/// [`Note`]: phichain_chart::note::Note
/// [`Highlighted`]: highlight::Highlighted
/// [`CurveNote`]: curve_note_track::CurveNote
/// [`CurveNoteTrack`]: curve_note_track::CurveNoteTrack
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameViewport(Rect::from_corners(Vec2::ZERO, Vec2::ZERO)))
            .insert_resource(ChartTime(0.0))
            .insert_resource(GameConfig::default())
            .insert_resource(Paused(true))
            .add_plugins(HighlightPlugin)
            .add_plugins(ScalePlugin)
            .add_plugins(CoreGamePlugin)
            .add_plugins(CurveNoteTrackPlugin)
            .add_plugins(ShapePlugin)
            .add_plugins(HitEffectPlugin)
            .add_plugins(ScorePlugin)
            .add_plugins(GameUiPlugin)
            .add_plugins(IllustrationPlugin);
    }
}
