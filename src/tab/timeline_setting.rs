use bevy::prelude::*;
use egui::Ui;

use super::timeline::TimelineSettings;

pub fn timeline_setting_tab(
    In(ui): In<&mut Ui>,
    mut timeline_settings: ResMut<TimelineSettings>,
) {
    egui::Grid::new("timeline_setting_grid")
        .num_columns(2)
        .spacing([40.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Zoom");
            ui.add(egui::DragValue::new(&mut timeline_settings.zoom).speed(0.01));
            ui.end_row();

            ui.label("Density");
            ui.add(egui::DragValue::new(&mut timeline_settings.density).speed(1));
            ui.end_row();
        });
}
