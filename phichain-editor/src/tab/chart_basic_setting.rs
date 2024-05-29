use crate::editing::command::meta::{EditMeta, EditOffset};
use crate::editing::command::EditorCommand;
use crate::editing::DoCommandEvent;
use crate::project::Project;
use crate::ui::latch;
use bevy::prelude::*;
use egui::Ui;
use phichain_chart::offset::Offset;

pub fn chart_basic_setting_tab(
    In(ui): In<&mut Ui>,
    mut offset: ResMut<Offset>,
    mut project: ResMut<Project>,

    mut event_writer: EventWriter<DoCommandEvent>,
) {
    egui::Grid::new("chart_basic_setting_grid")
        .num_columns(2)
        .spacing([20.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            let result = latch::latch(
                ui,
                "chart-basic-settings",
                (project.meta.clone(), offset.0),
                |ui| {
                    let mut finished = false;

                    ui.label(t!("tab.chart_basic_setting.offset"));
                    let response = ui.add(egui::DragValue::new(&mut offset.0).speed(1));
                    finished |= response.drag_stopped() || response.lost_focus();
                    ui.end_row();

                    ui.label(t!("tab.chart_basic_setting.name"));
                    let response = ui.text_edit_singleline(&mut project.meta.name);
                    finished |= response.drag_stopped() || response.lost_focus();
                    ui.end_row();

                    ui.label(t!("tab.chart_basic_setting.level"));
                    let response = ui.text_edit_singleline(&mut project.meta.level);
                    finished |= response.drag_stopped() || response.lost_focus();
                    ui.end_row();

                    ui.label(t!("tab.chart_basic_setting.composer"));
                    let response = ui.text_edit_singleline(&mut project.meta.composer);
                    finished |= response.drag_stopped() || response.lost_focus();
                    ui.end_row();

                    ui.label(t!("tab.chart_basic_setting.charter"));
                    let response = ui.text_edit_singleline(&mut project.meta.charter);
                    finished |= response.drag_stopped() || response.lost_focus();
                    ui.end_row();

                    ui.label(t!("tab.chart_basic_setting.illustrator"));
                    let response = ui.text_edit_singleline(&mut project.meta.illustrator);
                    finished |= response.drag_stopped() || response.lost_focus();
                    ui.end_row();

                    finished
                },
            );

            if let Some((meta_from, offset_from)) = result {
                if meta_from != project.meta {
                    event_writer.send(DoCommandEvent(EditorCommand::EditMeta(EditMeta::new(
                        meta_from,
                        project.meta.clone(),
                    ))));
                }

                if offset_from != offset.0 {
                    event_writer.send(DoCommandEvent(EditorCommand::EditOffset(EditOffset::new(
                        offset_from,
                        offset.0,
                    ))));
                }
            }
        });
}
