use bevy::prelude::*;
use bevy_egui::EguiContext;
use rfd::FileDialog;

use crate::{
    file::{pick_folder, PickingEvent, PickingKind},
    project::{project_not_loaded, LoadProjectEvent},
};

pub struct HomePlugin;

impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_system.run_if(project_not_loaded()))
            .add_systems(Update, load_project_system.run_if(project_not_loaded()));
    }
}

fn ui_system(world: &mut World) {
    let egui_context = world.query::<&mut EguiContext>().single_mut(world);
    let mut egui_context = egui_context.clone();
    let ctx = egui_context.get_mut();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(format!("phichain v{}", env!("CARGO_PKG_VERSION")));

        if ui.button("Load Project").clicked() {
            pick_folder(world, PickingKind::OpenProject, FileDialog::new());
        }
    });
}

fn load_project_system(
    mut picking_events: EventReader<PickingEvent>,
    mut events: EventWriter<LoadProjectEvent>,
) {
    for PickingEvent { path, kind } in picking_events.read() {
        if !matches!(kind, PickingKind::OpenProject) {
            continue;
        }
        if let Some(root_dir) = path {
            events.send(LoadProjectEvent(root_dir.to_path_buf()));
        }
    }
}
