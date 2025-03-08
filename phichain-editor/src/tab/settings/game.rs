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
                t!("tab.settings.category.game.fc_ap_indicator"),
                Some("是否启用 FC/AP 指示器。编辑器不含判定，即勾选后判定线恒定为黄色，不勾选则恒定为白色"),
                |ui| {
                    let response = ui.checkbox(&mut settings.game.fc_ap_indicator, "");
                    response.changed()
                },
            );

            ui.separator();

            finished |= ui.item(
                t!("tab.settings.category.game.hide_hit_effect"),
                Some("是否隐藏打击特效"),
                |ui| {
                    let response = ui.checkbox(&mut settings.game.hide_hit_effect, "");
                    response.changed()
                },
            );

            ui.separator();

            finished |= ui.item(
                t!("tab.settings.category.game.note_scale"),
                Some("音符的缩放比例"),
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
                t!("tab.settings.category.game.multi_highlight"),
                Some("是否开启多押高亮，即高亮所有等时音符"),
                |ui| {
                    let response = ui.checkbox(&mut settings.game.multi_highlight, "");
                    response.changed()
                },
            );

            #[cfg(debug_assertions)]
            {
                ui.separator();

                finished |= ui.item(
                    t!("tab.settings.category.game.hit_effect_follow_game_time"),
                    Some("打击特效是否跟随游戏时间。启用后，打击特效的渲染将不再基于游戏全局时间，而是基于谱面时间。仅在调试环境中存在"),
                    |ui| {
                        let response = ui.checkbox(&mut settings.game.hit_effect_follow_game_time, "");
                        response.changed()
                    },
                );
            }

            finished
        })
        .is_some()
    }
}
