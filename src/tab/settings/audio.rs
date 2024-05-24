use crate::settings::EditorSettings;
use crate::tab::settings::SettingCategory;
use crate::ui::latch;
use egui::Ui;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Audio;

impl SettingCategory for Audio {
    fn name(&self) -> &str {
        "tab.settings.category.audio.title"
    }

    fn ui(&self, ui: &mut Ui, settings: &mut EditorSettings) -> bool {
        egui::Grid::new("audio-settings-grid")
            .num_columns(2)
            .spacing([20.0, 2.0])
            .striped(true)
            .show(ui, |ui| {
                latch::latch(ui, "audio-settings", settings.general, |ui| {
                    let mut finished = false;
                    ui.label(t!("tab.settings.category.audio.music_volume"));
                    let response = ui.add(
                        egui::DragValue::new(&mut settings.audio.music_volume)
                            .clamp_range(0.00..=1.2)
                            .speed(0.01),
                    );
                    finished |= response.drag_stopped() || response.lost_focus();
                    ui.end_row();

                    ui.label(t!("tab.settings.category.audio.hit_sound_volume"));
                    let response = ui.add(
                        egui::DragValue::new(&mut settings.audio.hit_sound_volume)
                            .clamp_range(0.00..=1.2)
                            .speed(0.01),
                    );
                    finished |= response.drag_stopped() || response.lost_focus();
                    ui.end_row();

                    finished
                })
                .is_some()
            })
            .inner
    }
}
