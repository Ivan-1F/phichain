use crate::ui::compat::mute_keyboard_for_bevy_when_egui_wants_system;
use bevy::prelude::*;

mod compat;
pub mod latch;
pub mod utils;
pub mod widgets;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            mute_keyboard_for_bevy_when_egui_wants_system
                .after(bevy_egui::systems::process_input_system)
                .before(bevy_egui::EguiSet::BeginFrame),
        );
    }
}
