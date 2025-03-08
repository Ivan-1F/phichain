use crate::settings::EditorSettings;
use crate::tab::settings::{SettingCategory, SettingUi};
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
        latch::latch(ui, "game-settings", settings.game.clone(), |ui| {
            let mut finished = false;

            finished |= ui.item(
                t!("tab.settings.category.game.fc_ap_indicator.label"),
                Some(t!("tab.settings.category.game.fc_ap_indicator.description")),
                |ui| {
                    let response = ui.checkbox(&mut settings.game.fc_ap_indicator, "");
                    response.changed()
                },
            );

            ui.separator();

            finished |= ui.item(
                t!("tab.settings.category.game.hide_hit_effect.label"),
                Some(t!("tab.settings.category.game.hide_hit_effect.description")),
                |ui| {
                    let response = ui.checkbox(&mut settings.game.hide_hit_effect, "");
                    response.changed()
                },
            );

            ui.separator();

            finished |= ui.item(
                t!("tab.settings.category.game.note_scale.label"),
                Some(t!("tab.settings.category.game.note_scale.description")),
                |ui| {
                    let response = ui.add(
                        egui::DragValue::new(&mut settings.game.note_scale)
                            .range(0.50..=1.5)
                            .speed(0.01),
                    );
                    response.drag_stopped() || response.lost_focus()
                },
            );

            ui.separator();

            finished |= ui.item(
                t!("tab.settings.category.game.multi_highlight.label"),
                Some(t!("tab.settings.category.game.multi_highlight.description")),
                |ui| {
                    let response = ui.checkbox(&mut settings.game.multi_highlight, "");
                    response.changed()
                },
            );

            #[cfg(debug_assertions)]
            {
                ui.separator();

                finished |= ui.item(
                    t!("tab.settings.category.game.hit_effect_follow_game_time.label"),
                    Some(t!(
                        "tab.settings.category.game.hit_effect_follow_game_time.description"
                    )),
                    |ui| {
                        let response =
                            ui.checkbox(&mut settings.game.hit_effect_follow_game_time, "");
                        response.changed()
                    },
                );
            }

            finished
        })
        .is_some()
    }
}
