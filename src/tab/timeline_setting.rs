use bevy::prelude::*;
use egui::Ui;

use super::timeline::TimelineSettings;

pub fn timeline_setting_tab(In(ui): In<&mut Ui>, mut timeline_settings: ResMut<TimelineSettings>) {
    egui::Grid::new("timeline_setting_grid")
        .num_columns(2)
        .spacing([40.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label(t!("tab.timeline_setting.zoom"));
            ui.add(
                egui::DragValue::new(&mut timeline_settings.zoom)
                    .clamp_range(0.1..=f32::MAX)
                    .speed(0.01),
            );
            ui.end_row();

            ui.label(t!("tab.timeline_setting.density"));
            ui.add(
                egui::DragValue::new(&mut timeline_settings.density)
                    .clamp_range(1..=32)
                    .speed(1),
            );
            ui.end_row();

            ui.label(t!("tab.timeline_setting.lane"));
            ui.add(
                egui::DragValue::new(&mut timeline_settings.lanes)
                    .clamp_range(1..=32)
                    .speed(1),
            );
            ui.end_row();
        });
}
