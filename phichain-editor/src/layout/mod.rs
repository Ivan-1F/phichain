mod apply;
mod create;
mod delete;
mod rename;
pub mod ui_state;
mod update;

use crate::action::{ActionRegistrationExt, ActionRegistry};
use crate::identifier::Identifier;
use crate::layout::apply::{apply_layout_observer, ApplyLayout};
use crate::layout::create::{create_layout_observer, NewLayout};
use crate::layout::delete::{delete_layout_observer, DeleteLayout};
use crate::layout::rename::{rename_layout_observer, RenameLayout};
use crate::layout::ui_state::UiState;
use crate::layout::update::{update_layout_observer, UpdateLayout};
use crate::misc::WorkingDirectory;
use crate::notification::ToastsExt;
use crate::project::project_loaded;
use crate::ui::sides::SidesExt;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_egui::{EguiContext, EguiPrimaryContextPass};
use bevy_persistent::{Persistent, StorageFormat};
use egui_dock::DockState;
use phichain_game::GameSet;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const LAYOUT_SAVE_CHECK_INTERVAL: Duration = Duration::from_secs(3);

type Layout = DockState<Identifier>;

#[derive(Clone, Serialize, Deserialize)]
pub struct LayoutPreset {
    name: String,
    layout: Layout,
}

#[derive(Clone, Default, Resource, Serialize, Deserialize)]
struct LayoutPresetManager {
    presets: Vec<LayoutPreset>,
}

pub struct LayoutPlugin;

impl Plugin for LayoutPlugin {
    fn build(&self, app: &mut App) {
        let config_dir = app
            .world()
            .resource::<WorkingDirectory>()
            .config()
            .expect("Failed to locate config directory");

        let layout_presets = Persistent::<LayoutPresetManager>::builder()
            .name("Layout Presets")
            .format(StorageFormat::Json)
            .path(config_dir.join("layouts.json"))
            .default(LayoutPresetManager::default())
            .build()
            .expect("Failed to initialize layouts presets");

        let layout_path = config_dir.join("editor_layout.json");

        // try to load layout from file, fallback to default
        let ui_state = if layout_path.exists() {
            match std::fs::read_to_string(&layout_path)
                .ok()
                .and_then(|s| serde_json::from_str::<UiState>(&s).ok())
            {
                Some(state) => {
                    info!("Loaded editor layout from file");
                    state
                }
                None => {
                    warn!("Failed to load layout file, deleting and using default");
                    let _ = std::fs::remove_file(&layout_path);
                    UiState::default()
                }
            }
        } else {
            UiState::default()
        };

        app.insert_resource(layout_presets)
            .insert_resource(ui_state.clone())
            .insert_resource(LastSavedLayout(Some(ui_state)))
            .add_action(
                "phichain.save_layout_preset",
                |mut commands: Commands| {
                    commands.spawn(NewLayoutDialog::default());
                    Ok(())
                },
                None,
            )
            .add_systems(
                Update,
                save_layout_system
                    .run_if(project_loaded())
                    .run_if(on_timer(LAYOUT_SAVE_CHECK_INTERVAL)),
            )
            .add_systems(EguiPrimaryContextPass, modal_ui_system.in_set(GameSet))
            .add_observer(create_layout_observer)
            .add_observer(apply_layout_observer)
            .add_observer(delete_layout_observer)
            .add_observer(rename_layout_observer)
            .add_observer(update_layout_observer);
    }
}

#[derive(Resource, Default)]
struct LastSavedLayout(Option<UiState>);

fn save_layout_system(
    ui_state: Res<UiState>,
    working_dir: Res<WorkingDirectory>,
    mut last_saved: ResMut<LastSavedLayout>,
) {
    // check if layout actually changed by comparing serialized bytes
    // not using `.is_change()`: UiState gets mutably dereferenced every frame to render the UI, so `.is_change()` is always true
    // not using `==`: UiState does not implement `PartialEq`
    let layout_changed = match &last_saved.0 {
        None => true,
        Some(last) => match (serde_json::to_vec(last), serde_json::to_vec(&*ui_state)) {
            (Ok(a), Ok(b)) => a != b,
            _ => false,
        },
    };

    if layout_changed {
        let Ok(config_dir) = working_dir.config() else {
            return;
        };
        let layout_path = config_dir.join("editor_layout.json");

        match serde_json::to_string(&*ui_state) {
            Ok(json) => {
                if let Err(e) = std::fs::write(layout_path, json) {
                    error!("Failed to save editor layout: {}", e);
                } else {
                    debug!("Saved editor layout");
                    last_saved.0 = Some(ui_state.clone());
                }
            }
            Err(e) => error!("Failed to serialize layout: {}", e),
        }
    }
}

pub fn layout_menu(ui: &mut egui::Ui, world: &mut World) {
    let presets = world
        .resource::<Persistent<LayoutPresetManager>>()
        .presets
        .to_vec();

    ui.menu_button(t!("menu_bar.layout.title"), |ui| {
        if ui.button(t!("menu_bar.layout.default")).clicked() {
            world.trigger(ApplyLayout(UiState::default().state));
            ui.close_menu();
        }

        ui.separator();

        for (index, preset) in presets.iter().enumerate() {
            ui.menu_button(&preset.name, |ui| {
                if ui.button(t!("menu_bar.layout.item.apply")).clicked() {
                    world.trigger(ApplyLayout(preset.layout.clone()));
                    ui.close_menu();
                }
                if ui.button(t!("menu_bar.layout.item.rename")).clicked() {
                    world.spawn(RenameLayoutDialog {
                        index,
                        name: preset.name.clone(),
                    });
                    ui.close_menu();
                }
                if ui.button(t!("menu_bar.layout.item.update")).clicked() {
                    world.trigger(UpdateLayout(index));
                    ui.close_menu();
                }

                ui.separator();

                if ui.button(t!("menu_bar.layout.item.delete")).clicked() {
                    world.trigger(DeleteLayout(index));
                    ui.close_menu();
                }
            });
        }

        if !presets.is_empty() {
            ui.separator();
        }

        if ui
            .button(t!("action.phichain.save_layout_preset"))
            .clicked()
        {
            world.resource_scope(|world, mut actions: Mut<ActionRegistry>| {
                actions.run_action(world, "phichain.save_layout_preset");
            });
            ui.close_menu();
        }
    });
}

#[derive(Default, Debug, Clone, Component)]
struct NewLayoutDialog(String);

#[derive(Default, Debug, Clone, Component)]
struct RenameLayoutDialog {
    index: usize,
    name: String,
}

fn modal_ui_system(
    mut commands: Commands,
    mut context: Query<&mut EguiContext>,
    mut new_query: Query<(Entity, &mut NewLayoutDialog)>,
    mut rename_query: Query<(Entity, &mut RenameLayoutDialog)>,
    mut toasts: ResMut<crate::notification::ToastsStorage>,
) -> Result<()> {
    let Ok(egui_context) = context.single_mut() else {
        return Ok(());
    };

    let mut egui_context = egui_context.clone();
    let ctx = egui_context.get_mut();

    if let Ok((entity, mut dialog)) = new_query.single_mut() {
        let response = egui::Modal::new("new-layout".into()).show(ctx, |ui| {
            ui.heading(t!("menu_bar.layout.dialog.new.title"));
            ui.separator();

            ui.label(t!("menu_bar.layout.dialog.new.name_label"));
            let text_edit_response = ui.text_edit_singleline(&mut dialog.0);
            text_edit_response.request_focus();

            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

            ui.add_space(2.0);

            ui.sides(
                |_| {},
                |ui| {
                    if ui.button(t!("menu_bar.layout.dialog.new.save")).clicked() || enter_pressed {
                        if dialog.0.trim().is_empty() {
                            toasts.error(t!("menu_bar.layout.messages.empty_name"));
                        } else {
                            commands.trigger(NewLayout(dialog.0.clone()));
                            commands.entity(entity).despawn();
                        }
                    }
                    if ui.button(t!("menu_bar.layout.dialog.new.cancel")).clicked() {
                        commands.entity(entity).despawn();
                    }
                },
            );
        });

        if response.should_close() {
            commands.entity(entity).despawn();
        }
    }

    if let Ok((entity, mut dialog)) = rename_query.single_mut() {
        let response = egui::Modal::new("Rename Layout".into()).show(ctx, |ui| {
            ui.heading(t!("menu_bar.layout.dialog.rename.title"));
            ui.separator();

            ui.label(t!("menu_bar.layout.dialog.rename.name_label"));
            let text_edit_response = ui.text_edit_singleline(&mut dialog.name);
            text_edit_response.request_focus();

            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));

            ui.add_space(2.0);

            ui.sides(
                |_| {},
                |ui| {
                    if ui
                        .button(t!("menu_bar.layout.dialog.rename.save"))
                        .clicked()
                        || enter_pressed
                    {
                        if dialog.name.trim().is_empty() {
                            toasts.error(t!("menu_bar.layout.messages.empty_name"));
                        } else {
                            commands.trigger(RenameLayout {
                                index: dialog.index,
                                name: dialog.name.clone(),
                            });
                            commands.entity(entity).despawn();
                        }
                    }
                    if ui
                        .button(t!("menu_bar.layout.dialog.rename.cancel"))
                        .clicked()
                    {
                        commands.entity(entity).despawn();
                    }
                },
            );
        });

        if response.should_close() {
            commands.entity(entity).despawn();
        }
    }

    Ok(())
}
