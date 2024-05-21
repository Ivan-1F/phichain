use crate::audio::AudioSettings;
use crate::notification::{ToastsExt, ToastsStorage};
use bevy::prelude::*;
use bevy_persistent::Persistent;
use egui::Ui;

pub fn audio_setting_tab(
    In(ui): In<&mut Ui>,
    mut audio_settings: ResMut<Persistent<AudioSettings>>,
    mut toasts: ResMut<ToastsStorage>,
) {
    egui::Grid::new("audio_setting_grid")
        .num_columns(2)
        .spacing([20.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label(t!("tab.audio_setting.music_volume"));
            let response = ui.add(
                egui::DragValue::new(&mut audio_settings.music_volume)
                    .clamp_range(0.00..=1.2)
                    .speed(0.01),
            );
            if response.drag_stopped() || response.lost_focus() {
                if let Err(error) = audio_settings.persist() {
                    toasts.error(t!("audio_setting.save.failed", error = error));
                }
            }
            ui.end_row();

            ui.label(t!("tab.audio_setting.hit_sound_volume"));
            let response = ui.add(
                egui::DragValue::new(&mut audio_settings.hit_sound_volume)
                    .clamp_range(0.00..=1.2)
                    .speed(0.01),
            );
            if response.drag_stopped() || response.lost_focus() {
                if let Err(error) = audio_settings.persist() {
                    toasts.error(t!("audio_setting.save.failed", error = error));
                }
            }
            ui.end_row();
        });
}
