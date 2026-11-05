use crate::timeline::container::{ordered_timelines, push_timeline, swap_timelines};
use crate::timeline::event::EventTimeline;
use crate::timeline::kind::timeline_name;
use crate::timeline::note::NoteTimeline;
use crate::timeline::settings::TimelineSettings;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use egui::{RichText, Ui};
use phichain_chart::line::Line;

use super::timeline::NoteSideFilter;

pub fn timeline_setting_tab(In(mut ui): In<Ui>, world: &mut World) {
    let mut state: SystemState<(Query<(&Line, Entity)>, ResMut<TimelineSettings>, Commands)> =
        SystemState::new(world);
    let (line_query, mut timeline_settings, mut commands) = state.get_mut(world);

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

            ui.label(t!("tab.timeline_setting.show_spectrogram"));
            ui.checkbox(&mut timeline_settings.show_spectrogram, "");
            ui.end_row();

            ui.label(t!("tab.timeline_setting.spectrogram_opacity"));
            ui.add(
                egui::DragValue::new(&mut timeline_settings.spectrogram_opacity)
                    .range(0.0..=1.0)
                    .speed(0.01),
            );
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
                        commands.queue(|world: &mut World| {
                            push_timeline(world, NoteTimeline, None);
                        });
                        ui.close();
                    }
                    ui.separator();
                    for (line, entity) in line_query.iter() {
                        // TODO: move timeline selector to dedicated widget
                        if ui.button(&line.name).clicked() {
                            commands.queue(move |world: &mut World| {
                                push_timeline(world, NoteTimeline, Some(entity));
                            });
                            ui.close();
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
                        commands.queue(|world: &mut World| {
                            push_timeline(world, EventTimeline, None);
                        });
                        ui.close();
                    }
                    ui.separator();
                    for (line, entity) in line_query.iter() {
                        if ui.button(&line.name).clicked() {
                            commands.queue(move |world: &mut World| {
                                push_timeline(world, EventTimeline, Some(entity));
                            });
                            ui.close();
                        }
                    }
                },
            );
        });

        ui.end_row();

        state.apply(world);

        let timelines = ordered_timelines(world);
        let mut delete = None;
        let mut move_up = None;
        let mut move_down = None;

        for (index, timeline) in timelines.iter().enumerate() {
            let name = timeline_name(world, *timeline);
            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    if ui.button(" × ").clicked() {
                        delete.replace(*timeline);
                    }
                    ui.add_enabled_ui(index > 0, |ui| {
                        if ui.button(" ↑ ").clicked() {
                            move_up.replace(index);
                        }
                    });
                    ui.add_enabled_ui(index < timelines.len() - 1, |ui| {
                        if ui.button(" ↓ ").clicked() {
                            move_down.replace(index);
                        }
                    });
                    ui.label(name);
                });
            });
        }

        if let Some(delete) = delete {
            world.despawn(delete);
        }
        if let Some(move_up) = move_up {
            swap_timelines(world, timelines[move_up - 1], timelines[move_up]);
        }
        if let Some(move_down) = move_down {
            swap_timelines(world, timelines[move_down], timelines[move_down + 1]);
        }
    }
}
