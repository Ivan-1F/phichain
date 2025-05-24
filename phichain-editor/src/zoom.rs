use crate::action::ActionRegistrationExt;
use crate::hotkey::modifier::Modifier;
use crate::hotkey::Hotkey;
use crate::tab::game::GameCamera;
use bevy::app::App;
use bevy::prelude::*;

pub struct ZoomPlugin;

impl Plugin for ZoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_action(
            "phichain.zoom_in",
            zoom_in_system,
            Some(Hotkey::new(KeyCode::Equal, vec![Modifier::Control])),
        )
        .add_action(
            "phichain.zoom_out",
            zoom_out_system,
            Some(Hotkey::new(KeyCode::Minus, vec![Modifier::Control])),
        )
        .add_action(
            "phichain.reset_zoom",
            reset_zoom_system,
            Some(Hotkey::new(KeyCode::Digit0, vec![Modifier::Control])),
        );
    }
}

fn zoom_in_system(mut query: Query<&mut Projection, With<GameCamera>>) -> Result {
    let mut projection = query.single_mut()?;
    if let Projection::Orthographic(ref mut projection) = projection.as_mut() {
        projection.scale /= 1.25;
    }

    Ok(())
}

fn zoom_out_system(mut query: Query<&mut Projection, With<GameCamera>>) -> Result {
    let mut projection = query.single_mut()?;
    if let Projection::Orthographic(ref mut projection) = projection.as_mut() {
        projection.scale *= 1.25;
    }

    Ok(())
}

fn reset_zoom_system(mut query: Query<&mut Projection, With<GameCamera>>) -> Result {
    let mut projection = query.single_mut()?;
    if let Projection::Orthographic(ref mut projection) = projection.as_mut() {
        projection.scale = 1.0;
    }

    Ok(())
}
