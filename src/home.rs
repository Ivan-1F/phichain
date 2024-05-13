use std::path::PathBuf;

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_egui::EguiContext;
use futures_lite::future;
use rfd::FileDialog;

use crate::project::{project_not_loaded, LoadProjectEvent};

pub struct HomePlugin;

impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_system.run_if(project_not_loaded()))
            .add_systems(Update, load_project_system.run_if(project_not_loaded()));
    }
}

#[derive(Component)]
struct SelectedFolder(Task<Option<PathBuf>>);

fn ui_system(world: &mut World) {
    let egui_context = world.query::<&mut EguiContext>().single_mut(world);
    let mut egui_context = egui_context.clone();
    let ctx = egui_context.get_mut();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(format!("phichain v{}", env!("CARGO_PKG_VERSION")));

        if ui.button("Load Project").clicked() {
            let thread_pool = AsyncComputeTaskPool::get();
            let task = thread_pool.spawn(async move { FileDialog::new().pick_folder() });
            world.spawn(SelectedFolder(task));
        }
    });
}

fn load_project_system(mut commands: Commands, mut tasks: Query<(Entity, &mut SelectedFolder)>, mut events: EventWriter<LoadProjectEvent>) {
    for (entity, mut selected_folder) in &mut tasks {
        if let Some(result) = future::block_on(future::poll_once(&mut selected_folder.0)) {
            commands.entity(entity).despawn();
            if let Some(root_dir) = result {
                events.send(LoadProjectEvent(root_dir));
            }
        }
    }
}
