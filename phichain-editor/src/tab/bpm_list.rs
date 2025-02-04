use crate::editing::command::bpm_list::{CreateBpmPoint, EditBpmPoint, RemoveBpmPoint};
use crate::editing::command::EditorCommand;
use crate::editing::DoCommandEvent;
use crate::ui::latch;
use crate::ui::widgets::beat_value::BeatValue;
use bevy::prelude::*;
use egui::{ScrollArea, Ui};
use phichain_chart::beat;
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::{BpmList, BpmPoint};

pub fn bpm_list_tab(
    In(mut ui): In<Ui>,
    mut bpm_list: ResMut<BpmList>,
    mut event_writer: EventWriter<DoCommandEvent>,
) {
    let mut changes = Vec::new();
    let mut deletes = Vec::new();

    ScrollArea::vertical().show(&mut ui, |ui| {
        for index in 0..bpm_list.0.len() {
            let previous_beat = (index > 0)
                .then(|| bpm_list.0.get(index - 1).map(|x| x.beat))
                .flatten();
            let next_beat = bpm_list.0.get(index + 1).map(|x| x.beat);
            let point = bpm_list.0.get_mut(index).unwrap();

            ui.horizontal_top(|ui| {
                egui::Grid::new(format!("bpm_list_grid_{}", index))
                    .num_columns(2)
                    .spacing([20.0, 2.0])
                    .striped(true)
                    .show(ui, |ui| {
                        let result =
                            latch::latch(ui, format!("bpm_point_{}", index), *point, |ui| {
                                let mut finished = false;
                                let mut beat = point.beat;

                                ui.label(t!("tab.bpm_list.point.beat"));
                                ui.add_enabled_ui(point.beat != Beat::ZERO, |ui| {
                                    let start = previous_beat
                                        .map(|x| x + beat!(0, 1, 32))
                                        .unwrap_or(Beat::MIN);
                                    let range = start..=next_beat.unwrap_or(Beat::MAX);
                                    let response = ui
                                        .add(BeatValue::new(&mut beat).range(range))
                                        .on_disabled_hover_text(t!(
                                            "tab.bpm_list.zero_beat_not_editable"
                                        ));
                                    finished |= response.drag_stopped() || response.lost_focus();
                                });
                                ui.end_row();

                                ui.label(t!("tab.bpm_list.point.bpm"));
                                let response = ui.add(
                                    egui::DragValue::new(&mut point.bpm).range(0.01..=f32::MAX),
                                );
                                finished |= response.drag_stopped() || response.lost_focus();
                                ui.end_row();

                                if beat != point.beat {
                                    point.beat = beat;
                                    changes.push((index, beat));
                                }

                                finished
                            });

                        if let Some(from) = result {
                            if from != *point {
                                event_writer.send(DoCommandEvent(EditorCommand::EditBpmPoint(
                                    EditBpmPoint::new(index, from, *point),
                                )));
                            }
                        }
                    });

                ui.add_space(10.0);
                ui.add_enabled_ui(point.beat != Beat::ZERO, |ui| {
                    if ui
                        .button(" × ")
                        .on_disabled_hover_text(t!("tab.bpm_list.zero_beat_not_editable"))
                        .clicked()
                    {
                        deletes.push(index);
                    }
                });
            });

            ui.separator();
        }

        if ui.button(t!("tab.bpm_list.new")).clicked() {
            // takes the beat of last bpm point and add one beat or use Beat::ONE
            let beat = bpm_list
                .0
                .last()
                .map(|x| x.beat + Beat::ONE)
                .unwrap_or(Beat::ONE);
            event_writer.send(DoCommandEvent(EditorCommand::CreateBpmPoint(
                CreateBpmPoint::new(BpmPoint::new(beat, 120.0)),
            )));
        }
    });

    // recompute after all changes are applied
    if !changes.is_empty() {
        bpm_list.compute();
    }

    if !deletes.is_empty() {
        for i in deletes {
            event_writer.send(DoCommandEvent(EditorCommand::RemoveBpmPoint(
                RemoveBpmPoint::new(i),
            )));
        }
        bpm_list.compute();
    }
}
