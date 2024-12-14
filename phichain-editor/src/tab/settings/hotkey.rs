use crate::hotkey::next::{HotkeyRegistry, HotkeyState};
use crate::settings::EditorSettings;
use crate::tab::settings::SettingCategory;
use bevy::prelude::World;
use egui::Ui;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Hotkey;

impl SettingCategory for Hotkey {
    fn name(&self) -> &str {
        "tab.settings.category.hotkey.title"
    }

    fn ui(&self, ui: &mut Ui, _: &mut EditorSettings, world: &mut World) -> bool {
        // TODO: currently due to HashMap based registry, display is in arbitrary order
        // TODO: add grouping for hotkeys
        let registry = world.resource::<HotkeyRegistry>();
        let state = world.resource::<HotkeyState>();

        egui::Grid::new("audio-settings-grid")
            .num_columns(3)
            .spacing([20.0, 2.0])
            .striped(true)
            .show(ui, |ui| {
                for (id, default) in registry.0.iter() {
                    ui.label(t!(format!("hotkey.{}", id).as_str()));
                    let hotkey = state.get(id.clone()).unwrap_or(default.clone());
                    ui.label(hotkey.to_string());

                    ui.horizontal(|ui| {
                        // TODO
                        let _ = ui.button(t!("tab.settings.category.hotkey.record"));
                        let _ = ui.button(t!("tab.settings.category.hotkey.reset"));
                    });

                    ui.end_row();
                }
            });

        // handle hotkey config persist here, there's nothing to do with EditorSettings
        false
    }
}
