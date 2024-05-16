use crate::audio::AudioSettings;
use bevy::prelude::*;
use egui::Ui;

use crate::translation::Translator;

pub fn audio_setting_tab(
    In(ui): In<&mut Ui>,
    mut audio_settings: ResMut<AudioSettings>,
    translator: Translator,
) {
    egui::Grid::new("audio_setting_grid")
        .num_columns(2)
        .spacing([40.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label(translator.tr("tab.audio_setting.music_volume"));
            ui.add(
                egui::DragValue::new(&mut audio_settings.music_volume)
                    .clamp_range(0.00..=1.2)
                    .speed(0.01),
            );
            ui.end_row();

            ui.label(translator.tr("tab.audio_setting.hit_sound_volume"));
            ui.add(
                egui::DragValue::new(&mut audio_settings.hit_sound_volume)
                    .clamp_range(0.00..=1.2)
                    .speed(0.01),
            );
            ui.end_row();
        });
}
