use crate::audio::Offset;
use bevy::prelude::*;
use egui::Ui;
use rust_i18n::t;

pub fn chart_basic_setting_tab(In(ui): In<&mut Ui>, mut offset: ResMut<Offset>) {
    egui::Grid::new("chart_basic_setting_grid")
        .num_columns(2)
        .spacing([40.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label(t!("tab.chart_basic_setting.offset"));
            ui.add(egui::DragValue::new(&mut offset.0).speed(1));
            ui.end_row();
        });
}
