use bevy::prelude::*;
use egui::Ui;
use phichain_chart::offset::Offset;
use phichain_chart::project::Project;

pub fn chart_basic_setting_tab(In(mut ui): In<Ui>, offset: Res<Offset>, project: Res<Project>) {
    ui.disable();
    let mut offset = *offset;
    let mut meta = project.meta.clone();
    egui::Grid::new("chart_basic_setting_grid")
        .num_columns(2)
        .spacing([20.0, 2.0])
        .striped(true)
        .show(&mut ui, |ui| {
            ui.label(t!("tab.chart_basic_setting.offset"));
            ui.add(egui::DragValue::new(&mut offset.0).speed(1));
            ui.end_row();

            ui.label(t!("tab.chart_basic_setting.name"));
            ui.text_edit_singleline(&mut meta.name);
            ui.end_row();

            ui.label(t!("tab.chart_basic_setting.level"));
            ui.text_edit_singleline(&mut meta.level);
            ui.end_row();

            ui.label(t!("tab.chart_basic_setting.composer"));
            ui.text_edit_singleline(&mut meta.composer);
            ui.end_row();

            ui.label(t!("tab.chart_basic_setting.charter"));
            ui.text_edit_singleline(&mut meta.charter);
            ui.end_row();

            ui.label(t!("tab.chart_basic_setting.illustrator"));
            ui.text_edit_singleline(&mut meta.illustrator);
            ui.end_row();
        });
}
