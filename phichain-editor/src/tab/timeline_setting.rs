use crate::timeline::event::EventTimeline;
use crate::timeline::note::NoteTimeline;
use crate::timeline::settings::TimelineSettings;
use crate::timeline::TimelineItem;
use bevy::prelude::*;
use egui::{RichText, Ui};
use phichain_chart::line::Line;

use super::timeline::NoteSideFilter;

pub fn timeline_setting_tab(
    In(mut ui): In<Ui>,
    mut timeline_settings: ResMut<TimelineSettings>,
    line_query: Query<(&Line, Entity)>,
) {
    egui::Grid::new("timeline_setting_grid")
        .num_columns(2)
        .spacing([20.0, 2.0])
        .striped(true)
        .show(&mut ui, |ui| {
            ui.label(t!("tab.timeline_setting.zoom"));
            ui.add(
                egui::DragValue::new(&mut timeline_settings.zoom)
                    .range(0.1..=f32::MAX)
                    .speed(0.01),
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
                            .container
                            .push_right(TimelineItem::Note(NoteTimeline::new_binding()));
                        ui.close_menu();
                    }
                    ui.separator();
                    for (line, entity) in line_query.iter() {
                        // TODO: move timeline selector to dedicated widget
                        if ui.button(&line.name).clicked() {
                            timeline_settings
                                .container
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
                            .container
                            .push_right(TimelineItem::Event(EventTimeline::new_binding()));
                        ui.close_menu();
                    }
                    ui.separator();
                    for (line, entity) in line_query.iter() {
                        if ui.button(&line.name).clicked() {
                            timeline_settings
                                .container
                                .push_right(TimelineItem::Event(EventTimeline::new(entity)));
                            ui.close_menu();
                        }
                    }
                },
            );
        });

        ui.end_row();

        let container = &mut timeline_settings.container;
        let mut delete = None;
        let mut move_up = None;
        let mut move_down = None;

        for (index, timeline) in container.timelines.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    if ui.button(" × ").clicked() {
                        delete.replace(index);
                    }
                    ui.add_enabled_ui(index > 0, |ui| {
                        if ui.button(" ↑ ").clicked() {
                            move_up.replace(index);
                        }
                    });
                    ui.add_enabled_ui(index < container.timelines.len() - 1, |ui| {
                        if ui.button(" ↓ ").clicked() {
                            move_down.replace(index);
                        }
                    });
                    let label = match &timeline.timeline {
                        TimelineItem::Note(timeline) => match timeline.0 {
                            None => t!("tab.timeline_setting.timelines.note_timeline.binding"),
                            Some(entity) => t!(
                                "tab.timeline_setting.timelines.note_timeline.for_line",
                                line = line_query.get(entity).unwrap().0.name,
                            ),
                        },
                        TimelineItem::Event(timeline) => match timeline.0 {
                            None => t!("tab.timeline_setting.timelines.event_timeline.binding"),
                            Some(entity) => t!(
                                "tab.timeline_setting.timelines.event_timeline.for_line",
                                line = line_query.get(entity).unwrap().0.name,
                            ),
                        },
                    };
                    ui.label(label);
                });
            });
        }

        if let Some(delete) = delete {
            container.remove(delete);
        }

        if let Some(move_up) = move_up {
            container.swap(move_up - 1, move_up);
        }

        if let Some(move_down) = move_down {
            container.swap(move_down, move_down + 1);
        }
    }
}
