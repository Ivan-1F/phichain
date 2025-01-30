use crate::action::ActionRegistrationExt;
use crate::hotkey::modifier::Modifier;
use crate::hotkey::Hotkey;
use crate::tab::game::GameCamera;
use bevy::app::App;
use bevy::prelude::{KeyCode, OrthographicProjection, Plugin, Query, With};

pub struct ZoomPlugin;

impl Plugin for ZoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_action(
            "phichain.zoom_in",
            |mut query: Query<&mut OrthographicProjection, With<GameCamera>>| {
                let mut projection = query.single_mut();
                projection.scale /= 1.25;
            },
            Some(Hotkey::new(KeyCode::Equal, vec![Modifier::Control])),
        )
        .add_action(
            "phichain.zoom_out",
            |mut query: Query<&mut OrthographicProjection, With<GameCamera>>| {
                let mut projection = query.single_mut();
                projection.scale *= 1.25;
            },
            Some(Hotkey::new(KeyCode::Minus, vec![Modifier::Control])),
        )
        .add_action(
            "phichain.reset_zoom",
            |mut query: Query<&mut OrthographicProjection, With<GameCamera>>| {
                let mut projection = query.single_mut();
                projection.scale = 1.0;
            },
            Some(Hotkey::new(KeyCode::Digit0, vec![Modifier::Control])),
        );
    }
}
