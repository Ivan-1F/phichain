use std::path::PathBuf;

use bevy::prelude::*;
use bevy_egui::EguiContext;
use rfd::FileDialog;

use crate::{
    file::{pick_file, pick_folder, PickingEvent, PickingKind},
    notification::{ToastsExt, ToastsStorage},
    project::{create_project, project_not_loaded, LoadProjectEvent, ProjectMeta},
};

#[derive(Resource, Debug, Default)]
pub struct CreateProjectForm {
    meta: ProjectMeta,
    music: Option<PathBuf>,
    illustration: Option<PathBuf>,
}

pub struct HomePlugin;

impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CreateProjectForm::default())
            .add_systems(Update, ui_system.run_if(project_not_loaded()))
            .add_systems(Update, load_project_system.run_if(project_not_loaded()))
            .add_systems(
                Update,
                (
                    handle_select_illustration_system,
                    handle_select_music_system,
                    handle_create_project_system,
                )
                    .run_if(project_not_loaded()),
            );
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

        ui.separator();

        ui.horizontal(|ui| {
            let form = world.resource_mut::<CreateProjectForm>();
            ui.label(format!("{:?}", form.music));
            if ui.button("Select Music").clicked() {
                pick_file(
                    world,
                    PickingKind::SelectMusic,
                    FileDialog::new().add_filter("Music", &["wav", "mp3", "ogg", "flac"]),
                );
            }
        });
        ui.horizontal(|ui| {
            let form = world.resource_mut::<CreateProjectForm>();
            ui.label(format!("{:?}", form.illustration));
            if ui.button("Select Illustration").clicked() {
                pick_file(
                    world,
                    PickingKind::SelectIllustration,
                    FileDialog::new().add_filter("Illustration", &["png", "jpg", "jpeg"]),
                );
            }
        });

        egui::Grid::new("project_meta_grid")
            .num_columns(2)
            .spacing([40.0, 2.0])
            .striped(true)
            .show(ui, |ui| {
                let mut form = world.resource_mut::<CreateProjectForm>();
                ui.label("Name");
                ui.text_edit_singleline(&mut form.meta.name);
                ui.end_row();

                ui.label("Level");
                ui.text_edit_singleline(&mut form.meta.level);
                ui.end_row();

                ui.label("Composer");
                ui.text_edit_singleline(&mut form.meta.composer);
                ui.end_row();

                ui.label("Charter");
                ui.text_edit_singleline(&mut form.meta.charter);
                ui.end_row();

                ui.label("Illustrator");
                ui.text_edit_singleline(&mut form.meta.illustrator);
                ui.end_row();
            });

        let form = world.resource_mut::<CreateProjectForm>();
        if ui.button("Create Project").clicked() {
            if form.music.is_none() {
                let mut toasts = world.resource_mut::<ToastsStorage>();
                toasts.error("Music is not selected");
                return;
            };
            if form.illustration.is_none() {
                let mut toasts = world.resource_mut::<ToastsStorage>();
                toasts.error("Illustration is not selected");
                return;
            };

            pick_folder(world, PickingKind::CreateProject, FileDialog::new());
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

fn handle_select_illustration_system(
    mut events: EventReader<PickingEvent>,
    mut form: ResMut<CreateProjectForm>,
) {
    for PickingEvent { path, kind } in events.read() {
        if !matches!(kind, PickingKind::SelectIllustration) {
            continue;
        }
        form.illustration.clone_from(path);
    }
}

fn handle_select_music_system(
    mut events: EventReader<PickingEvent>,
    mut form: ResMut<CreateProjectForm>,
) {
    for PickingEvent { path, kind } in events.read() {
        if !matches!(kind, PickingKind::SelectMusic) {
            continue;
        }
        form.music.clone_from(path);
    }
}

fn handle_create_project_system(
    mut events: EventReader<PickingEvent>,
    form: Res<CreateProjectForm>,
    mut load_project_events: EventWriter<LoadProjectEvent>,

    mut toasts: ResMut<ToastsStorage>,
) {
    for PickingEvent { path, kind } in events.read() {
        if !matches!(kind, PickingKind::CreateProject) {
            continue;
        }

        let Some(root_path) = path else {
            return;
        };

        let Some(ref music_path) = form.music else {
            return;
        };

        let Some(ref illustration_path) = form.illustration else {
            return;
        };

        match create_project(
            root_path.clone(),
            music_path.clone(),
            illustration_path.clone(),
            form.meta.clone(),
        ) {
            Ok(_) => {
                load_project_events.send(LoadProjectEvent(root_path.clone()));
            }
            Err(error) => toasts.error(format!("{:?}", error)),
        }
    }
}
