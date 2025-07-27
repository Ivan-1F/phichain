use bevy::app::App;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

pub struct ImeCompatPlugin;

impl Plugin for ImeCompatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_ime_system);
    }
}

fn update_ime_system(
    mut contexts: EguiContexts,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    window_query.single_mut()?.ime_enabled = ctx.wants_keyboard_input();

    Ok(())
}
