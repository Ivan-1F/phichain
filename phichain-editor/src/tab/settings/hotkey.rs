use crate::action::ActionRegistry;
use crate::hotkey::record::RecordingHotkey;
use crate::hotkey::HotkeyContext;
use crate::identifier::Identifier;
use crate::settings::EditorSettings;
use crate::tab::settings::{SettingCategory, SettingUi};
use bevy::ecs::system::SystemState;
use bevy::prelude::{Entity, Res, World};
use egui::Ui;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Hotkey;

impl SettingCategory for Hotkey {
    fn name(&self) -> &str {
        "tab.settings.category.hotkey.title"
    }

    fn ui(&self, ui: &mut Ui, _: &mut EditorSettings, world: &mut World) -> bool {
        let mut state: SystemState<(HotkeyContext, Res<ActionRegistry>)> = SystemState::new(world);
        let (mut ctx, actions) = state.get_mut(world);

        let mut despawn = None::<Entity>;
        let mut spawn = None::<Identifier>;

        for (id, default) in ctx.registry.0.clone().iter() {
            let key = if actions.0.contains_key(id) {
                format!("action.{}", id)
            } else {
                format!("hotkey.{}", id)
            };

            ui.item(t!(key.as_str()), None::<&str>, |ui| {
                // the order is revered since we use [`egui::Sides`] for `ui.item`. the right part starts from the end
                ui.horizontal(|ui| {
                    // Record and reset buttons
                    ui.add_enabled_ui(
                        ctx.state
                            .get(id.clone())
                            .is_some_and(|x| x != default.clone()),
                        |ui| {
                            if ui
                                .button(t!("tab.settings.category.hotkey.reset"))
                                .clicked()
                            {
                                ctx.state.set(id.clone(), default.clone());
                                let _ = ctx.save();
                            }
                        },
                    );

                    match ctx.query.get_single() {
                        Ok((recording, entity)) if recording.id == *id => {
                            if ui
                                .button(t!("tab.settings.category.hotkey.cancel"))
                                .clicked()
                            {
                                despawn.replace(entity);
                            }
                        }
                        _ => {
                            ui.add_enabled_ui(ctx.query.get_single().is_err(), |ui| {
                                if ui
                                    .button(t!("tab.settings.category.hotkey.record"))
                                    .clicked()
                                {
                                    spawn.replace(id.clone());
                                }
                            });
                        }
                    }

                    match ctx.query.get_single() {
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
                            let hotkey = ctx.state.get(id.clone()).unwrap_or(default.clone());
                            ui.label(hotkey.to_string());
                        }
                    }
                });

                false
            });

            ui.separator();
        }

        if let Some(entity) = despawn {
            world.entity_mut(entity).despawn();
        }
        if let Some(id) = spawn {
            world.spawn(RecordingHotkey::new(id));
        }

        // handle hotkey config persist here, there's nothing to do with EditorSettings
        false
    }
}
