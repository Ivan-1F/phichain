use crate::timeline::settings::TimelineSettings;
use bevy::prelude::*;
use egui::Ui;

use super::timeline::NoteSideFilter;

pub fn timeline_setting_tab(In(mut ui): In<Ui>, world: &mut World) {
    let mut timeline_settings = world.resource_mut::<TimelineSettings>();

    egui::Grid::new("timeline_setting_grid")
        .num_columns(2)
        .spacing([20.0, 2.0])
        .striped(true)
        .show(&mut ui, |ui| {
            ui.label(t!("tab.timeline_setting.zoom"));
            ui.add(
                egui::DragValue::new(&mut timeline_settings.zoom)
                    .range(0.1..=5.0)
                    .speed(0.02),
            );
            ui.end_row();

            ui.label(t!("tab.timeline_setting.density"));
            ui.add(
                egui::DragValue::new(&mut timeline_settings.density)
                    .range(1..=32)
                    .speed(1),
            );
            ui.end_row();

            ui.label(t!("tab.timeline_setting.lane"));
            ui.add(
                egui::DragValue::new(&mut timeline_settings.lanes)
                    .range(1..=64)
                    .speed(1),
            );
            ui.end_row();

            ui.label(t!("tab.timeline_setting.note_side_filter.title"));
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut timeline_settings.note_side_filter,
                    NoteSideFilter::All,
                    t!("tab.timeline_setting.note_side_filter.all"),
                );
                ui.selectable_value(
                    &mut timeline_settings.note_side_filter,
                    NoteSideFilter::Above,
                    t!("tab.timeline_setting.note_side_filter.above"),
                );
                ui.selectable_value(
                    &mut timeline_settings.note_side_filter,
                    NoteSideFilter::Below,
                    t!("tab.timeline_setting.note_side_filter.below"),
                );
            });
            ui.end_row();

            ui.label(t!("tab.timeline_setting.show_waveform"));
            ui.checkbox(&mut timeline_settings.show_waveform, "");
            ui.end_row();
        });
}
