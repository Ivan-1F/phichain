use crate::ui::compat::mute_keyboard_for_bevy_when_egui_wants_system;
use bevy::prelude::*;
use bevy_egui::EguiPreUpdateSet;

mod compat;
pub mod edit;
pub mod latch;
pub mod sides;
pub mod widgets;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                edit::interrupt_gesture_system,
                mute_keyboard_for_bevy_when_egui_wants_system,
            )
                .chain()
                .after(EguiPreUpdateSet::ProcessInput),
        );
    }
}
