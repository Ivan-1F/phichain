use crate::chart::beat::Beat;
use crate::timing::{BpmList, BpmPoint};
use crate::widgets::beat_value::BeatValue;
use bevy::prelude::*;
use egui::Ui;
use num::Rational32;

pub fn bpm_list_tab(In(ui): In<&mut Ui>, mut bpm_list: ResMut<BpmList>) {
    let mut changes = Vec::new();
    let mut deletes = Vec::new();

    for index in 0..bpm_list.0.len() {
        let previous_beat = (index > 0)
            .then(|| bpm_list.0.get(index - 1).map(|x| x.beat))
            .flatten();
        let next_beat = bpm_list.0.get(index + 1).map(|x| x.beat);
        let point = bpm_list.0.get_mut(index).unwrap();

        ui.horizontal_top(|ui| {
            egui::Grid::new(format!("audio_setting_grid_{}", index))
                .num_columns(2)
                .spacing([40.0, 2.0])
                .striped(true)
                .show(ui, |ui| {
                    let mut beat = point.beat;

                    ui.label(t!("tab.bpm_list.point.beat"));
                    ui.add_enabled_ui(point.beat != Beat::ZERO, |ui| {
                        let start = previous_beat
                            .map(|x| x + Beat::new(0, Rational32::new(1, 32)))
                            .unwrap_or(Beat::MIN);
                        let range = start..=next_beat.unwrap_or(Beat::MAX);
                        ui.add(BeatValue::new(&mut beat).clamp_range(range))
                            .on_disabled_hover_text(t!("tab.bpm_list.zero_beat_not_editable"));
                    });
                    ui.end_row();

                    ui.label(t!("tab.bpm_list.point.bpm"));
                    ui.add(egui::DragValue::new(&mut point.bpm).clamp_range(0.01..=f32::MAX));
                    ui.end_row();

                    if beat != point.beat {
                        point.beat = beat;
                        changes.push((index, beat));
                    }
                });

            ui.add_space(20.0);
            ui.add_enabled_ui(point.beat != Beat::ZERO, |ui| {
                if ui
                    .button("×")
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
        bpm_list.0.push(BpmPoint::new(beat, 120.0));
        bpm_list.compute();
    }

    // recompute after all changes are applied
    if !changes.is_empty() {
        bpm_list.compute();
    }

    if !deletes.is_empty() {
        for i in deletes {
            bpm_list.0.remove(i);
        }
        bpm_list.compute();
    }
}
