use crate::settings::EditorSettings;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use egui::Ui;

pub fn quick_action_tab(
    In(ui): In<&mut Ui>,
    mut editor_settings: ResMut<Persistent<EditorSettings>>,
) {
    ui.horizontal(|ui| {
        ui.label(t!("tab.settings.category.audio.playback_rate"));
        ui.add(
            egui::DragValue::new(&mut editor_settings.audio.playback_rate)
                .suffix("x")
                .clamp_range(0.01..=1.0)
                .speed(0.01),
        );
    });
}
