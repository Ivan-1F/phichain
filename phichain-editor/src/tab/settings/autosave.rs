use crate::settings::EditorSettings;
use crate::tab::settings::{SettingCategory, SettingUi};
use crate::ui::latch;
use bevy::prelude::*;
use egui::Ui;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct AutoSave;

impl SettingCategory for AutoSave {
    fn name(&self) -> &str {
        "tab.settings.category.autosave.title"
    }

    fn ui(&self, ui: &mut Ui, settings: &mut EditorSettings, _world: &mut World) -> bool {
        ui.label(t!("tab.settings.category.autosave.note"));
        ui.separator();

        latch::latch(ui, "autosave-settings", settings.autosave, |ui| {
            let mut finished = false;
            finished |= ui.item(
                t!("tab.settings.category.autosave.enabled"),
                None::<&str>,
                |ui| ui.checkbox(&mut settings.autosave.enabled, "").changed(),
            );

            ui.separator();

            ui.add_enabled_ui(settings.autosave.enabled, |ui| {
                finished |= ui.item(
                    t!("tab.settings.category.autosave.interval.label"),
                    Some(t!("tab.settings.category.autosave.interval.description")),
                    |ui| {
                        let response = ui.add(
                            egui::DragValue::new(&mut settings.autosave.interval_secs)
                                .range(10.0..=600.0)
                                .speed(1.0)
                                .suffix(t!("tab.settings.category.autosave.interval.suffix")),
                        );
                        response.drag_stopped() || response.lost_focus()
                    },
                );

                ui.separator();

                finished |= ui.item(
                    t!("tab.settings.category.autosave.idle_delay.label"),
                    Some(t!("tab.settings.category.autosave.idle_delay.description")),
                    |ui| {
                        let response = ui.add(
                            egui::DragValue::new(&mut settings.autosave.idle_delay_secs)
                                .range(1.0..=30.0)
                                .speed(1.0)
                                .suffix(t!("tab.settings.category.autosave.idle_delay.suffix")),
                        );
                        response.drag_stopped() || response.lost_focus()
                    },
                );

                ui.separator();

                finished |= ui.item(
                    t!("tab.settings.category.autosave.max_backups.label"),
                    Some(t!("tab.settings.category.autosave.max_backups.description")),
                    |ui| {
                        let response = ui.add(
                            egui::DragValue::new(&mut settings.autosave.max_backup_count)
                                .range(1..=20)
                                .speed(1.0)
                                .suffix(t!("tab.settings.category.autosave.max_backups.suffix")),
                        );
                        response.drag_stopped() || response.lost_focus()
                    },
                );
            });

            finished
        })
        .is_some()
    }
}
