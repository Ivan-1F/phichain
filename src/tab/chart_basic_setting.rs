use crate::audio::Offset;
use crate::project::Project;
use bevy::prelude::*;
use egui::Ui;

pub fn chart_basic_setting_tab(
    In(ui): In<&mut Ui>,
    mut offset: ResMut<Offset>,
    mut project: ResMut<Project>,
) {
    egui::Grid::new("chart_basic_setting_grid")
        .num_columns(2)
        .spacing([40.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label(t!("tab.chart_basic_setting.offset"));
            ui.add(egui::DragValue::new(&mut offset.0).speed(1));
            ui.end_row();

            ui.label(t!("tab.chart_basic_setting.name"));
            ui.text_edit_singleline(&mut project.meta.name);
            ui.end_row();

            ui.label(t!("tab.chart_basic_setting.level"));
            ui.text_edit_singleline(&mut project.meta.level);
            ui.end_row();

            ui.label(t!("tab.chart_basic_setting.composer"));
            ui.text_edit_singleline(&mut project.meta.composer);
            ui.end_row();

            ui.label(t!("tab.chart_basic_setting.charter"));
            ui.text_edit_singleline(&mut project.meta.charter);
            ui.end_row();

            ui.label(t!("tab.chart_basic_setting.illustrator"));
            ui.text_edit_singleline(&mut project.meta.illustrator);
            ui.end_row();
        });
}
