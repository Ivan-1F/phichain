use crate::timeline::event::EventTimeline;
use crate::timeline::note::NoteTimeline;
use crate::timeline::settings::TimelineSettings;
use crate::timeline::TimelineItem;
use bevy::prelude::*;
use egui::Ui;
use phichain_chart::line::Line;

use super::timeline::NoteSideFilter;

pub fn timeline_setting_tab(
    In(ui): In<&mut Ui>,
    mut timeline_settings: ResMut<TimelineSettings>,
    line_query: Query<(&Line, Entity)>,
) {
    egui::Grid::new("timeline_setting_grid")
        .num_columns(2)
        .spacing([20.0, 2.0])
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

            ui.label("Enable Multi-Line Edit");
            ui.checkbox(&mut timeline_settings.multi_line_editing, "");
            ui.end_row();
        });

    if timeline_settings.multi_line_editing {
        ui.separator();
        ui.columns(2, |columns| {
            columns[0].menu_button("New Note Timeline", |ui| {
                for (index, (_, entity)) in line_query.iter().enumerate() {
                    // TODO: use a readable identifier for this (e.g. name)
                    // TODO: move timeline selector to dedicated widget
                    if ui.button(format!("Line #{}", index)).clicked() {
                        for (_, percent) in &mut timeline_settings.timelines {
                            *percent /= 1.2;
                        }
                        timeline_settings
                            .timelines
                            .push((TimelineItem::Note(NoteTimeline::new(entity)), 1.0));
                        ui.close_menu();
                    }
                }
            });
            columns[1].menu_button("New Event Timeline", |ui| {
                for (index, (_, entity)) in line_query.iter().enumerate() {
                    // TODO: use a readable identifier for this (e.g. name)
                    if ui.button(format!("Line #{}", index)).clicked() {
                        for (_, percent) in &mut timeline_settings.timelines {
                            *percent /= 1.2;
                        }
                        timeline_settings
                            .timelines
                            .push((TimelineItem::Event(EventTimeline::new(entity)), 1.0));
                        ui.close_menu();
                    }
                }
            });
        });

        ui.end_row();

        let timelines = &mut timeline_settings.timelines;
        let mut deletes = vec![];

        for index in 0..timelines.len() {
            let prev = (index > 0)
                .then(|| timelines.get(index - 1).map(|x| x.1))
                .flatten();
            let next = timelines.get(index + 1).map(|x| x.1);
            let (timeline, percent) = timelines.get_mut(index).unwrap();

            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{:?}", timeline));
                    let start = prev.unwrap_or(0.0) + 0.05;
                    let end = next.map(|x| x - 0.05).unwrap_or(1.0);
                    ui.add(
                        egui::DragValue::new(percent)
                            .speed(0.005)
                            .clamp_range(start..=end),
                    );
                    if ui.button(" × ").clicked() {
                        deletes.push(index);
                    }
                });
            });
        }

        for index in deletes {
            timelines.remove(index);
        }
    }
}
