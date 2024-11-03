use super::GameCamera;
use crate::editing::pending::Pending;
use crate::project::project_loaded;
use crate::selection::{Selected, SelectedLine};
use crate::settings::EditorSettings;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use phichain_chart::line::Line;
use phichain_chart::note::Note;
use phichain_chart::project::Project;
use phichain_game::core::HoldComponent;
use phichain_game::GameConfig;

pub struct CoreGamePlugin;

impl Plugin for CoreGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, zoom_scale_system.run_if(project_loaded()))
            .add_systems(Update, sync_game_config_system.run_if(project_loaded()))
            .add_systems(Update, update_note_tint_system.run_if(project_loaded()))
            .add_systems(
                Update,
                sync_hold_components_tint_system
                    .after(update_note_tint_system)
                    .run_if(project_loaded()),
            )
            .add_systems(
                Update,
                update_line_tint_system
                    .after(phichain_game::core::update_line_system)
                    .run_if(project_loaded()),
            );
    }
}

fn zoom_scale_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut OrthographicProjection, With<GameCamera>>,
) {
    let mut projection = query.single_mut();
    if keyboard.pressed(KeyCode::KeyI) {
        projection.scale /= 1.01;
    } else if keyboard.pressed(KeyCode::KeyO) {
        projection.scale *= 1.01;
    }
}

fn update_note_tint_system(
    mut query: Query<(&mut Sprite, Option<&Selected>, Option<&Pending>), With<Note>>,
) {
    for (mut sprite, selected, pending) in &mut query {
        let tint = if selected.is_some() {
            Color::LIME_GREEN
        } else {
            Color::WHITE
        };
        let alpha = if pending.is_some() { 40.0 / 255.0 } else { 1.0 };
        sprite.color = tint.with_a(alpha);
    }
}

fn sync_hold_components_tint_system(
    mut component_query: Query<(&mut Sprite, &Parent), With<HoldComponent>>,
    parent_query: Query<&Sprite, Without<HoldComponent>>,
) {
    for (mut sprite, parent) in &mut component_query {
        if let Ok(parent_sprite) = parent_query.get(parent.get()) {
            sprite.color = parent_sprite.color;
        }
    }
}

fn sync_game_config_system(
    editor_settings: Res<Persistent<EditorSettings>>,
    project: Res<Project>,
    mut game_config: ResMut<GameConfig>,
) {
    game_config.note_scale = editor_settings.game.note_scale;
    game_config.fc_ap_indicator = editor_settings.game.fc_ap_indicator;
    game_config.multi_highlight = editor_settings.game.multi_highlight;
    game_config.hide_hit_effect = editor_settings.game.hide_hit_effect;
    game_config.hit_effect_follow_game_time = editor_settings.game.hit_effect_follow_game_time;
    game_config.name = project.meta.name.clone();
    game_config.level = project.meta.level.clone();
}

fn update_line_tint_system(
    mut query: Query<(&mut Sprite, Entity), With<Line>>,
    selected_line: Res<SelectedLine>,
) {
    for (mut sprite, entity) in &mut query {
        if entity == selected_line.0 {
            sprite.color = Color::LIME_GREEN.with_a(sprite.color.a());
        }
    }
}
