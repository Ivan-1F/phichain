use crate::editing::history::Edits;
use crate::ui::widgets::beat_value::BeatValue;
use bevy::prelude::*;
use egui::{ScrollArea, Ui};
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::BpmPoint;
use phichain_chart::id::BpmPointId;

pub fn bpm_list_tab(
    In(mut ui): In<Ui>,
    points: Query<(Entity, &BpmPointId, &BpmPoint)>,
    mut edits: Edits,
) {
    let mut points: Vec<_> = points.iter().collect();
    points.sort_by_key(|(_, id, point)| (point.beat, **id));
    ScrollArea::vertical().show(&mut ui, |ui| {
        for (entity, _, point) in &points {
            ui.push_id(*entity, |ui| {
                ui.horizontal_top(|ui| {
                    egui::Grid::new("bpm_point").num_columns(2).show(ui, |ui| {
                        let mut editor =
                            edits.component(*entity, *point, t!("history.edit_bpm_point"));
                        ui.label(t!("tab.bpm_list.point.beat"));
                        ui.add_enabled_ui(point.beat != Beat::ZERO, |ui| {
                            editor.edit(ui, "beat", |ui, point| {
                                ui.add(
                                    BeatValue::new(&mut point.beat)
                                        .range(phichain_chart::beat!(0, 1, i32::MAX)..=Beat::MAX),
                                );
                            });
                        })
                        .response
                        .on_disabled_hover_text(t!("tab.bpm_list.zero_beat_not_editable"));
                        ui.end_row();
                        ui.label(t!("tab.bpm_list.point.bpm"));
                        editor.edit(ui, "bpm", |ui, point| {
                            ui.add(egui::DragValue::new(&mut point.bpm).range(0.01..=f32::MAX));
                        });
                        ui.end_row();
                    });
                    if ui
                        .add_enabled(point.beat != Beat::ZERO, egui::Button::new(" × "))
                        .on_disabled_hover_text(t!("tab.bpm_list.zero_beat_not_editable"))
                        .clicked()
                    {
                        let entity = *entity;
                        edits.once(t!("history.delete_bpm_point"), move |commands| {
                            commands.entity(entity).despawn();
                        });
                    }
                });
            });
            ui.separator();
        }
        if ui.button(t!("tab.bpm_list.new")).clicked() {
            let beat = points
                .last()
                .map(|(_, _, point)| point.beat + Beat::ONE)
                .unwrap_or(Beat::ONE);
            edits.once(t!("history.create_bpm_point"), move |commands| {
                commands.spawn(BpmPoint::new(beat, 120.0));
            });
        }
    });
}
