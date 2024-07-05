use crate::timeline::event::EventTimeline;
use crate::timeline::note::NoteTimeline;
use crate::timeline::settings::TimelineSettings;
use crate::timeline::TimelineItem;
use bevy::prelude::*;
use egui::{RichText, Ui};
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
        });

    {
        ui.separator();
        ui.columns(2, |columns| {
            columns[0].menu_button(
                t!("tab.timeline_setting.timelines.new_note_timeline"),
                |ui| {
                    if ui
                        .button(
                            RichText::new(t!("tab.timeline_setting.timelines.binding")).strong(),
                        )
                        .clicked()
                    {
                        timeline_settings
                            .timelines_container
                            .push_right(TimelineItem::Note(NoteTimeline::new_binding()));
                        ui.close_menu();
                    }
                    ui.separator();
                    for (index, (_, entity)) in line_query.iter().enumerate() {
                        // TODO: use a readable identifier for this (e.g. name)
                        // TODO: move timeline selector to dedicated widget
                        if ui.button(format!("Line #{}", index)).clicked() {
                            timeline_settings
                                .timelines_container
                                .push_right(TimelineItem::Note(NoteTimeline::new(entity)));
                            ui.close_menu();
                        }
                    }
                },
            );
            columns[1].menu_button(
                t!("tab.timeline_setting.timelines.new_event_timeline"),
                |ui| {
                    if ui
                        .button(
                            RichText::new(t!("tab.timeline_setting.timelines.binding")).strong(),
                        )
                        .clicked()
                    {
                        timeline_settings
                            .timelines_container
                            .push_right(TimelineItem::Event(EventTimeline::new_binding()));
                        ui.close_menu();
                    }
                    ui.separator();
                    for (index, (_, entity)) in line_query.iter().enumerate() {
                        // TODO: use a readable identifier for this (e.g. name)
                        if ui.button(format!("Line #{}", index)).clicked() {
                            timeline_settings
                                .timelines_container
                                .push_right(TimelineItem::Event(EventTimeline::new(entity)));
                            ui.close_menu();
                        }
                    }
                },
            );
        });

        ui.end_row();

        let timelines = &mut timeline_settings.timelines_container;
        let mut deletes = vec![];

        for (index, timeline) in timelines.timelines.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{:?}", timeline));
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
