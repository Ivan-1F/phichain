use crate::ui::compat::mute_keyboard_for_bevy_when_egui_wants_system;
use bevy::prelude::*;
use bevy_egui::EguiPreUpdateSet;

mod compat;
pub mod latch;
pub mod widgets;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            mute_keyboard_for_bevy_when_egui_wants_system.after(EguiPreUpdateSet::ProcessInput),
        );
    }
}
