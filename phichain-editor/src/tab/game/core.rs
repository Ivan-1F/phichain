use super::GameCamera;
use crate::editing::pending::Pending;
use crate::project::project_loaded;
use crate::selection::Selected;
use bevy::prelude::*;
use phichain_chart::note::Note;
use phichain_game::core::HoldComponent;

pub struct CoreGamePlugin;

impl Plugin for CoreGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, zoom_scale_system.run_if(project_loaded()))
            .add_systems(PostUpdate, update_note_tint_system.run_if(project_loaded()))
            .add_systems(
                PostUpdate,
                sync_hold_components_tint_system
                    .after(update_note_tint_system)
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
