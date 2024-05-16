use crate::audio::Offset;
use bevy::prelude::*;
use egui::Ui;

use crate::translation::Translator;

pub fn chart_basic_setting_tab(
    In(ui): In<&mut Ui>,
    mut offset: ResMut<Offset>,
    translator: Translator,
) {
    egui::Grid::new("chart_basic_setting_grid")
        .num_columns(2)
        .spacing([40.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label(translator.tr("tab.chart_basic_setting.offset"));
            ui.add(egui::DragValue::new(&mut offset.0).speed(1));
            ui.end_row();
        });
}