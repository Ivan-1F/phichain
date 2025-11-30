use crate::action::{ActionRegistrationExt, ActionRegistry};
use crate::identifier::Identifier;
use crate::misc::WorkingDirectory;
use crate::ui::sides::SidesExt;
use crate::UiState;
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
            .add_observer(apply_layout_observer);
    }
}

pub fn layout_menu(ui: &mut egui::Ui, world: &mut World) {
    let presets = world
        .resource::<Persistent<LayoutPresetManager>>()
        .presets
        .iter()
        .cloned()
        .collect::<Vec<_>>();

    ui.menu_button("Layout", |ui| {
        let _ = ui.button("Default");

        ui.separator();

        for preset in &presets {
            ui.menu_button(&preset.name, |ui| {
                if ui.button("Apply").clicked() {
                    world.trigger(ApplyLayout(preset.layout.clone()));
                    ui.close_menu();
                }
                if ui.button("Rename").clicked() {
                    ui.close_menu();
                }
                if ui.button("Update from Current Layout").clicked() {
                    ui.close_menu();
                }

                ui.separator();

                if ui.button("Delete").clicked() {
                    ui.close_menu();
                }
            });
        }

        if !presets.is_empty() {
            ui.separator();
        }

        if ui.button("Save Current Layout").clicked() {
            world.resource_scope(|world, mut actions: Mut<ActionRegistry>| {
                actions.run_action(world, "phichain.new_layout");
            });
            ui.close_menu();
        }
    });
}

#[derive(Default, Debug, Clone, Component)]
struct NewLayoutDialog(String);

#[derive(Debug, Clone, Event)]
struct NewLayout(String);

fn modal_ui_system(
    mut commands: Commands,
    mut context: Query<&mut EguiContext>,
    mut query: Query<(Entity, &mut NewLayoutDialog)>,
) -> Result<()> {
    let Ok((entity, mut dialog)) = query.single_mut() else {
        return Ok(());
    };
    let Ok(egui_context) = context.single_mut() else {
        return Ok(());
    };

    let mut egui_context = egui_context.clone();
    let ctx = egui_context.get_mut();

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

    Ok(())
}

fn create_layout_observer(
    trigger: Trigger<NewLayout>,
    mut manager: ResMut<Persistent<LayoutPresetManager>>,
    ui_state: Res<UiState>,
) -> Result<()> {
    manager.presets.push(LayoutPreset {
        name: trigger.0.clone(),
        layout: ui_state.state.clone(),
    });

    manager.persist()?;

    Ok(())
}

#[derive(Debug, Clone, Event)]
struct ApplyLayout(Layout);

fn apply_layout_observer(
    trigger: Trigger<ApplyLayout>,
    mut ui_state: ResMut<UiState>,
) -> Result<()> {
    ui_state.state = trigger.0.clone();

    Ok(())
}
