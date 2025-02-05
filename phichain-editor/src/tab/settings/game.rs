use crate::settings::EditorSettings;
use crate::tab::settings::SettingCategory;
use crate::ui::latch;
use bevy::prelude::World;
use egui::Ui;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Game;

impl SettingCategory for Game {
    fn name(&self) -> &str {
        "tab.settings.category.game.title"
    }

    fn ui(&self, ui: &mut Ui, settings: &mut EditorSettings, _world: &mut World) -> bool {
        egui::Grid::new("game-settings-grid")
            .num_columns(2)
            .spacing([20.0, 2.0])
            .striped(true)
            .show(ui, |ui| {
                latch::latch(ui, "game-settings", settings.game.clone(), |ui| {
                    let mut finished = false;
                    ui.label(t!("tab.settings.category.game.fc_ap_indicator"));
                    let response = ui.checkbox(&mut settings.game.fc_ap_indicator, "");
                    finished |= response.changed();
                    ui.end_row();

                    ui.label(t!("tab.settings.category.game.hide_hit_effect"));
                    let response = ui.checkbox(&mut settings.game.hide_hit_effect, "");
                    finished |= response.changed();
                    ui.end_row();

                    ui.label(t!("tab.settings.category.game.note_scale"));
                    let response = ui.add(
                        egui::DragValue::new(&mut settings.game.note_scale)
                            .range(0.50..=1.5)
                            .speed(0.01),
                    );
                    finished |= response.drag_stopped() || response.lost_focus();
                    ui.end_row();

                    ui.label(t!("tab.settings.category.game.multi_highlight"));
                    let response = ui.checkbox(&mut settings.game.multi_highlight, "");
                    finished |= response.changed();
                    ui.end_row();

                    #[cfg(debug_assertions)]
                    {
                        ui.label(t!("tab.settings.category.game.hit_effect_follow_game_time"));
                        let response =
                            ui.checkbox(&mut settings.game.hit_effect_follow_game_time, "");
                        finished |= response.changed();
                        ui.end_row();
                    }

                    finished
                })
                .is_some()
            })
            .inner
    }
}
