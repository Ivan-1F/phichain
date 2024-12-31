use std::path::PathBuf;

use bevy::prelude::*;
use bevy_egui::EguiContext;
use bevy_persistent::Persistent;
use egui::{Align2, RichText, ScrollArea, Sense};
use rfd::FileDialog;

use crate::recent_projects::{PersistentRecentProjectsExt, RecentProjects};
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

/// Marker resource to control the visibility of the create project dialog
///
/// This should always be removed after sending [`LoadProjectEvent`]
#[derive(Resource, Debug, Default)]
pub struct CreatingProject;

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
                    .run_if(project_not_loaded().and_then(resource_exists::<CreateProjectForm>)),
            );
    }
}

fn ui_system(world: &mut World) {
    let Ok(egui_context) = world.query::<&mut EguiContext>().get_single_mut(world) else {
        return;
    };
    let mut egui_context = egui_context.clone();
    let ctx = egui_context.get_mut();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(format!("Phichain v{}", env!("CARGO_PKG_VERSION")));

        ui.separator();

        let mut open = world.contains_resource::<CreatingProject>();
        egui::Window::new(t!("home.create_project.label"))
            .collapsible(false)
            .resizable([true, false])
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut open)
            .show(ctx, |ui| {
                egui::Grid::new("create_project_grid")
                    .num_columns(2)
                    .spacing([20.0, 2.0])
                    .striped(true)
                    .show(ui, |ui| {
                        if ui.button(t!("home.create_project.select_music")).clicked() {
                            pick_file(
                                world,
                                PickingKind::SelectMusic,
                                FileDialog::new()
                                    .add_filter("Music", &["wav", "mp3", "ogg", "flac"]),
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
                                FileDialog::new()
                                    .add_filter("Illustration", &["png", "jpg", "jpeg"]),
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

                    pick_folder(world, PickingKind::CreateProject, FileDialog::new());
                }
            });

        if !open {
            world.remove_resource::<CreatingProject>();
        }

        ui.horizontal(|ui| {
            if ui.button(t!("home.open_project.load")).clicked() {
                pick_folder(world, PickingKind::OpenProject, FileDialog::new());
            }
            if ui.button(t!("home.create_project.create")).clicked() {
                world.insert_resource(CreatingProject);
            }
        });

        ui.separator();

        ui.style_mut().interaction.selectable_labels = false;

        let mut remove = None;
        let mut open = None;

        let mut recent_projects = world.resource_mut::<Persistent<RecentProjects>>();
        if recent_projects.0.is_empty() {
            ui.label(t!("home.recent_projects.empty"));
        }
        ScrollArea::vertical().show(ui, |ui| {
            for (index, recent_project) in recent_projects.0.iter().rev().enumerate() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .add(egui::Label::new(&recent_project.name).sense(Sense::click()))
                            .clicked()
                        {
                            open.replace(recent_project.path.clone());
                        }
                        ui.add_space(ui.available_width() - 10.0);
                        if ui
                            .add(egui::Label::new("×").sense(Sense::click()))
                            .clicked()
                        {
                            remove.replace(index);
                        }
                    });
                    ui.label(RichText::new(recent_project.path.to_string_lossy()).weak());
                    ui.label(
                        RichText::new(t!(
                            "home.recent_projects.last_opened",
                            time = recent_project.last_opened.format("%Y/%m/%d %H:%M")
                        ))
                        .weak(),
                    );
                })
                .response
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .on_hover_and_drag_cursor(egui::CursorIcon::PointingHand);
            }
        });
        ui.style_mut().interaction.selectable_labels = true;

        if let Some(index) = remove {
            recent_projects.remove(index);
        }

        if let Some(open) = open {
            world.send_event(LoadProjectEvent(open));
            world.remove_resource::<CreatingProject>();
        }
    });
}

fn load_project_system(
    mut commands: Commands,
    mut picking_events: EventReader<PickingEvent>,
    mut events: EventWriter<LoadProjectEvent>,
) {
    for PickingEvent { path, kind } in picking_events.read() {
        if !matches!(kind, PickingKind::OpenProject) {
            continue;
        }
        if let Some(root_dir) = path {
            events.send(LoadProjectEvent(root_dir.to_path_buf()));
            commands.remove_resource::<CreatingProject>();
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
    mut commands: Commands,
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

        match create_project(
            root_path.clone(),
            music_path.clone(),
            form.illustration.clone(),
            form.meta.clone(),
        ) {
            Ok(_) => {
                load_project_events.send(LoadProjectEvent(root_path.clone()));
                commands.remove_resource::<CreatingProject>();
            }
            Err(error) => toasts.error(format!("{:?}", error)),
        }
    }
}
