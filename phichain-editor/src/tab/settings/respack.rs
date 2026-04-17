use crate::misc::WorkingDirectory;
use crate::respack::{scan_respacks, ReloadRespack};
use crate::settings::EditorSettings;
use crate::tab::settings::{SettingCategory, SettingUi};
use bevy::prelude::World;
use egui::Ui;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Respack;

impl SettingCategory for Respack {
    fn name(&self) -> &str {
        "tab.settings.category.respack.title"
    }

    fn description(&self) -> &str {
        "tab.settings.category.respack.description"
    }

    fn ui(&self, ui: &mut Ui, settings: &mut EditorSettings, world: &mut World) -> bool {
        let packs = scan_respacks(world.resource::<WorkingDirectory>());
        let current = settings.game.respack.clone();
        let builtin_label = t!("tab.settings.category.respack.builtin");

        let changed = ui.item(
            t!("tab.settings.category.respack.pack.label"),
            Some(t!("tab.settings.category.respack.pack.description")),
            |ui| {
                let mut changed = false;
                let selected_text: &str = current.as_deref().unwrap_or(&builtin_label);
                egui::ComboBox::from_id_salt("respack-picker")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(current.is_none(), builtin_label.as_ref())
                            .clicked()
                            && current.is_some()
                        {
                            settings.game.respack = None;
                            changed = true;
                        }
                        for name in &packs {
                            let selected = current.as_deref() == Some(name.as_str());
                            if ui.selectable_label(selected, name).clicked() && !selected {
                                settings.game.respack = Some(name.clone());
                                changed = true;
                            }
                        }
                    });
                changed
            },
        );

        ui.separator();

        let reload_clicked = ui
            .item(
                t!("tab.settings.category.respack.reload.label"),
                Some(t!("tab.settings.category.respack.reload.description")),
                |ui| {
                    ui.button(t!("tab.settings.category.respack.reload.button"))
                        .clicked()
                },
            );

        // A selection change needs to both persist and trigger a reload.
        // A plain reload click doesn't modify settings — we trigger directly
        // and return `false` so the outer code skips the persist step.
        if changed || reload_clicked {
            world.trigger(ReloadRespack);
        }

        changed
    }
}
