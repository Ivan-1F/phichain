use crate::hotkey::next::{HotkeyRegistry, HotkeyState};
use crate::hotkey::record::RecordingHotkey;
use crate::misc::WorkingDirectory;
use crate::settings::EditorSettings;
use crate::tab::settings::SettingCategory;
use bevy::prelude::{Entity, Mut, World};
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

        world.resource_scope(|world, registry: Mut<HotkeyRegistry>| {
            world.resource_scope(|world, mut state: Mut<HotkeyState>| {
                world.resource_scope(|world, working_directory: Mut<WorkingDirectory>| {
                    let mut recording = world.query::<(&RecordingHotkey, Entity)>();

                    egui::Grid::new("hotkey-settings-grid")
                        .num_columns(3)
                        .spacing([20.0, 2.0])
                        .striped(true)
                        .show(ui, |ui| {
                            for (id, default) in registry.0.iter() {
                                ui.label(t!(format!("hotkey.{}", id).as_str()));
                                match recording.get_single(world) {
                                    Ok((recording, _)) if recording.id == *id => {
                                        let mut keys = recording
                                            .modifiers
                                            .iter()
                                            .map(|x| x.to_string())
                                            .collect::<Vec<_>>();

                                        if let Some(key) = recording.key {
                                            keys.push(format!("{:?}", key))
                                        }

                                        ui.label(if keys.is_empty() {
                                            t!("tab.settings.category.hotkey.recording").to_string()
                                        } else {
                                            keys.join(" + ")
                                        });
                                    }
                                    _ => {
                                        let hotkey =
                                            state.get(id.clone()).unwrap_or(default.clone());
                                        ui.label(hotkey.to_string());
                                    }
                                }

                                ui.horizontal(|ui| {
                                    match recording.get_single(world) {
                                        Ok((recording, entity)) if recording.id == *id => {
                                            if ui
                                                .button(t!("tab.settings.category.hotkey.cancel"))
                                                .clicked()
                                            {
                                                world.entity_mut(entity).despawn();
                                            }
                                        }
                                        _ => {
                                            ui.add_enabled_ui(
                                                recording.get_single(world).is_err(),
                                                |ui| {
                                                    if ui
                                                        .button(t!(
                                                            "tab.settings.category.hotkey.record"
                                                        ))
                                                        .clicked()
                                                    {
                                                        world.spawn(RecordingHotkey::new(
                                                            id.clone(),
                                                        ));
                                                    }
                                                },
                                            );
                                        }
                                    }

                                    ui.add_enabled_ui(
                                        state.get(id.clone()).is_some_and(|x| x != default.clone()),
                                        |ui| {
                                            if ui
                                                .button(t!("tab.settings.category.hotkey.reset"))
                                                .clicked()
                                            {
                                                state.set(id.clone(), default.clone());
                                                let _ = state.save_to(
                                                    working_directory
                                                        .config()
                                                        .expect("Failed to locate config directory")
                                                        .join("hotkey.yml"),
                                                );
                                            }
                                        },
                                    );
                                });

                                ui.end_row();
                            }
                        });
                });
            });
        });

        // handle hotkey config persist here, there's nothing to do with EditorSettings
        false
    }
}
