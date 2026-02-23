use crate::settings::EditorSettings;
use crate::tab::settings::{SettingCategory, SettingUi};
use crate::ui::latch;
use bevy::prelude::World;
use egui::Ui;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Graphics;

impl SettingCategory for Graphics {
    fn name(&self) -> &str {
        "tab.settings.category.graphics.title"
    }

    fn description(&self) -> &str {
        "tab.settings.category.graphics.description"
    }

    fn ui(&self, ui: &mut Ui, settings: &mut EditorSettings, _world: &mut World) -> bool {
        latch::latch(ui, "graphics-settings", settings.graphics.clone(), |ui| {
            let mut finished = false;

            finished |= ui.item(
                t!("tab.settings.category.graphics.ui_scale.label"),
                Some(t!("tab.settings.category.graphics.ui_scale.description")),
                |ui| {
                    ui.selectable_value(
                        &mut settings.graphics.ui_scale,
                        3.0,
                        t!("tab.settings.category.graphics.ui_scale.xl"),
                    )
                    .clicked()
                        || ui
                            .selectable_value(
                                &mut settings.graphics.ui_scale,
                                2.5,
                                t!("tab.settings.category.graphics.ui_scale.large"),
                            )
                            .clicked()
                        || ui
                            .selectable_value(
                                &mut settings.graphics.ui_scale,
                                2.0,
                                t!("tab.settings.category.graphics.ui_scale.base"),
                            )
                            .clicked()
                        || ui
                            .selectable_value(
                                &mut settings.graphics.ui_scale,
                                1.5,
                                t!("tab.settings.category.graphics.ui_scale.small"),
                            )
                            .clicked()
                        || ui
                            .selectable_value(
                                &mut settings.graphics.ui_scale,
                                1.0,
                                t!("tab.settings.category.graphics.ui_scale.xs"),
                            )
                            .clicked()
                },
            );

            ui.separator();

            finished |= ui.item(
                t!("tab.settings.category.graphics.vsync.label"),
                Some(t!("tab.settings.category.graphics.vsync.description")),
                |ui| {
                    let response = ui.checkbox(&mut settings.graphics.vsync, "");
                    response.changed()
                },
            );

            finished
        })
        .is_some()
    }
}
