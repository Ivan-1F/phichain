mod apply;
mod create;
mod delete;
mod rename;
mod update;

use crate::action::{ActionRegistrationExt, ActionRegistry};
use crate::identifier::Identifier;
use crate::layout::apply::{apply_layout_observer, ApplyLayout};
use crate::layout::create::{create_layout_observer, NewLayout};
use crate::layout::delete::{delete_layout_observer, DeleteLayout};
use crate::layout::rename::{rename_layout_observer, RenameLayout};
use crate::layout::update::{update_layout_observer, UpdateLayout};
use crate::misc::WorkingDirectory;
use crate::ui::sides::SidesExt;
use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiPrimaryContextPass};
use bevy_persistent::{Persistent, StorageFormat};
use egui_dock::DockState;
use phichain_game::GameSet;
use serde::{Deserialize, Serialize};

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

        let resource = Persistent::<LayoutPresetManager>::builder()
            .name("Editor Layouts")
            .format(StorageFormat::Json)
            .path(config_dir.join("layouts.json"))
            .default(LayoutPresetManager::default())
            .build()
            .expect("Failed to initialize editor layouts");

        app.insert_resource(resource)
            .add_action(
                "phichain.new_layout",
                |mut commands: Commands| {
                    commands.spawn(NewLayoutDialog::default());
                    Ok(())
                },
                None,
            )
            .add_systems(EguiPrimaryContextPass, modal_ui_system.in_set(GameSet))
            .add_observer(create_layout_observer)
            .add_observer(apply_layout_observer)
            .add_observer(delete_layout_observer)
            .add_observer(rename_layout_observer)
            .add_observer(update_layout_observer);
    }
}

pub fn layout_menu(ui: &mut egui::Ui, world: &mut World) {
    let presets = world
        .resource::<Persistent<LayoutPresetManager>>()
        .presets
        .to_vec();

    ui.menu_button(t!("menu_bar.layout.title"), |ui| {
        let _ = ui.button(t!("menu_bar.layout.default"));

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

        if ui.button(t!("menu_bar.layout.save_current")).clicked() {
            world.resource_scope(|world, mut actions: Mut<ActionRegistry>| {
                actions.run_action(world, "phichain.new_layout");
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
) -> Result<()> {
    let Ok(egui_context) = context.single_mut() else {
        return Ok(());
    };

    let mut egui_context = egui_context.clone();
    let ctx = egui_context.get_mut();

    if let Ok((entity, mut dialog)) = new_query.single_mut() {
        let response = egui::Modal::new("New Layout".into()).show(ctx, |ui| {
            ui.heading("New Layout");
            ui.separator();

            ui.label("Layout name:");
            ui.text_edit_singleline(&mut dialog.0).request_focus();

            ui.add_space(2.0);

            ui.sides(
                |_| {},
                |ui| {
                    if ui.button("Save").clicked() {
                        commands.trigger(NewLayout(dialog.0.clone()));
                        commands.entity(entity).despawn();
                    }
                    if ui.button("Cancel").clicked() {
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
            ui.heading("Rename Layout");
            ui.separator();

            ui.label("New name:");
            ui.text_edit_singleline(&mut dialog.name).request_focus();

            ui.add_space(2.0);

            ui.sides(
                |_| {},
                |ui| {
                    if ui.button("Save").clicked() {
                        commands.trigger(RenameLayout {
                            index: dialog.index,
                            name: dialog.name.clone(),
                        });
                        commands.entity(entity).despawn();
                    }
                    if ui.button("Cancel").clicked() {
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
