use crate::settings::EditorSettings;
use crate::tab::settings::{SettingCategory, SettingUi};
use crate::ui::latch;
use bevy::prelude::World;
use egui::Ui;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Audio;

impl SettingCategory for Audio {
    fn name(&self) -> &str {
        "tab.settings.category.audio.title"
    }

    fn ui(&self, ui: &mut Ui, settings: &mut EditorSettings, _world: &mut World) -> bool {
        latch::latch(ui, "audio-settings", settings.audio.clone(), |ui| {
            let mut finished = false;

            finished |= ui.item(
                t!("tab.settings.category.audio.music_volume.label"),
                Some(t!("tab.settings.category.audio.music_volume.description")),
                |ui| {
                    let response = ui.add(
                        egui::DragValue::new(&mut settings.audio.music_volume)
                            .range(0.00..=1.2)
                            .speed(0.01),
                    );
                    response.drag_stopped() || response.lost_focus()
                },
            );

            ui.separator();

            finished |= ui.item(
                t!("tab.settings.category.audio.hit_sound_volume.label"),
                Some(t!(
                    "tab.settings.category.audio.hit_sound_volume.description"
                )),
                |ui| {
                    let response = ui.add(
                        egui::DragValue::new(&mut settings.audio.hit_sound_volume)
                            .range(0.00..=1.2)
                            .speed(0.01),
                    );
                    response.drag_stopped() || response.lost_focus()
                },
            );

            ui.separator();

            finished |= ui.item(
                t!("tab.settings.category.audio.playback_rate.label"),
                Some(t!("tab.settings.category.audio.playback_rate.description")),
                |ui| {
                    let response = ui.add(
                        egui::DragValue::new(&mut settings.audio.playback_rate)
                            .range(0.01..=2.0)
                            .speed(0.01),
                    );
                    response.drag_stopped() || response.lost_focus()
                },
            );

            finished
        })
        .is_some()
    }
}
