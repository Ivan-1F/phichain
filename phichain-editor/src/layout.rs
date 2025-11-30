use crate::action::{ActionRegistrationExt, ActionRegistry};
use crate::identifier::Identifier;
use crate::misc::WorkingDirectory;
use crate::UiState;
use bevy::app::{App, Plugin};
use bevy::prelude::{Mut, Res, ResMut, Resource, World};
use bevy_persistent::{Persistent, StorageFormat};
use egui_dock::DockState;
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

        app.insert_resource(resource).add_action(
            "phichain.save_layout",
            |mut manager: ResMut<Persistent<LayoutPresetManager>>, ui_state: Res<UiState>| {
                manager.presets.push(LayoutPreset {
                    name: "New Layout".to_string(),
                    layout: ui_state.state.clone(),
                });

                manager.persist()?;

                Ok(())
            },
            None,
        );
    }
}

pub fn layout_menu(ui: &mut egui::Ui, world: &mut World) {
    let presets = world
        .resource::<Persistent<LayoutPresetManager>>()
        .presets
        .iter()
        .map(|preset| preset.name.clone())
        .collect::<Vec<_>>();

    ui.menu_button("Layout", |ui| {
        let _ = ui.button("Default");

        ui.separator();

        for preset in &presets {
            ui.menu_button(preset, |ui| {
                let _ = ui.button("Apply");
                let _ = ui.button("Rename");
                let _ = ui.button("Update from Current Layout");
                let _ = ui.separator();
                let _ = ui.button("Delete");
            });
        }

        if !presets.is_empty() {
            ui.separator();
        }

        if ui.button("Save Current Layout").clicked() {
            world.resource_scope(|world, mut actions: Mut<ActionRegistry>| {
                actions.run_action(world, "phichain.save_layout");
            })
        }
    });
}
