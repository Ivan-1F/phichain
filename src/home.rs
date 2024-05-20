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
        ui.heading(format!("Phichain v{}", env!("CARGO_PKG_VERSION")));

        ui.separator();

        ui.label(t!("home.open_project.label"));

        if ui.button(t!("home.open_project.load")).clicked() {
            pick_folder(world, PickingKind::OpenProject, FileDialog::new());
        }

        ui.separator();

        ui.label(t!("home.create_project.label"));

        egui::Grid::new("create_project_grid")
            .num_columns(2)
            .spacing([20.0, 2.0])
            .striped(true)
            .show(ui, |ui| {
                if ui.button(t!("home.create_project.select_music")).clicked() {
                    pick_file(
                        world,
                        PickingKind::SelectMusic,
                        FileDialog::new().add_filter("Music", &["wav", "mp3", "ogg", "flac"]),
                    );
                }
                let form = world.resource::<CreateProjectForm>();
                let music_path = match &form.music {
                    None => t!("home.create_project.unselected").to_string(),
                    Some(path) => path.display().to_string(),
                };
                ui.label(music_path);
                ui.end_row();

                if ui
                    .button(t!("home.create_project.select_illustration"))
                    .clicked()
                {
                    pick_file(
                        world,
                        PickingKind::SelectIllustration,
                        FileDialog::new().add_filter("Illustration", &["png", "jpg", "jpeg"]),
                    );
                }
                let form = world.resource::<CreateProjectForm>();
                let illustration_text = match &form.illustration {
                    None => t!("home.create_project.unselected").to_string(),
                    Some(path) => path.display().to_string(),
                };
                ui.label(illustration_text);
                ui.end_row();

                let mut form = world.resource_mut::<CreateProjectForm>();

                ui.label(t!("home.create_project.name"));
                ui.text_edit_singleline(&mut form.meta.name);
                ui.end_row();

                ui.label(t!("home.create_project.level"));
                ui.text_edit_singleline(&mut form.meta.level);
                ui.end_row();

                ui.label(t!("home.create_project.composer"));
                ui.text_edit_singleline(&mut form.meta.composer);
                ui.end_row();

                ui.label(t!("home.create_project.charter"));
                ui.text_edit_singleline(&mut form.meta.charter);
                ui.end_row();

                ui.label(t!("home.create_project.illustrator"));
                ui.text_edit_singleline(&mut form.meta.illustrator);
                ui.end_row();
            });

        let form = world.resource_mut::<CreateProjectForm>();
        if ui.button(t!("home.create_project.create")).clicked() {
            if form.music.is_none() {
                let mut toasts = world.resource_mut::<ToastsStorage>();
                toasts.error(t!("home.create_project.music_unselected"));
                return;
            };
            if form.illustration.is_none() {
                let mut toasts = world.resource_mut::<ToastsStorage>();
                toasts.error(t!("home.create_project.illustration_unselected"));
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
