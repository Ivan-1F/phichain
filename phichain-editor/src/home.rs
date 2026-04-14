use crate::recent_projects::{PersistentRecentProjectsExt, RecentProjects};
use crate::settings::EditorSettings;
use crate::tab::settings::settings_ui;
use crate::translation::Languages;
use crate::ui::sides::SidesExt;
use crate::ui::widgets::language_combobox::language_combobox;
use crate::{
    file::{pick_file, pick_folder, picking_event, FilePickingAppExt},
    notification::{ToastsExt, ToastsStorage},
    project::{create_project, project_not_loaded, LoadProjectEvent, ProjectMeta},
};

use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiPrimaryContextPass};
use bevy_persistent::Persistent;
use egui::{Color32, CursorIcon, Id, RichText, ScrollArea, Sense};

use phichain_game::loader::nonblocking::LoadingProject;
use rfd::FileDialog;
use std::path::PathBuf;

picking_event!(PickedProject);
picking_event!(PickedIllustration);
picking_event!(PickedMusic);
picking_event!(PickedCreateProject);

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

/// Marker resource to control the visibility of the settings screen
#[derive(Resource, Debug, Default)]
pub struct OpenSettings;

pub struct HomePlugin;

impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {
        app.register_picking_event::<PickedProject>()
            .register_picking_event::<PickedIllustration>()
            .register_picking_event::<PickedMusic>()
            .register_picking_event::<PickedCreateProject>()
            .insert_resource(CreateProjectForm::default())
            .add_systems(
                EguiPrimaryContextPass,
                ui_system.run_if(project_not_loaded()),
            )
            .add_observer(load_project_observer)
            .add_observer(handle_select_illustration_observer)
            .add_observer(handle_select_music_observer)
            .add_observer(handle_create_project_observer);
    }
}

fn ui_system(world: &mut World) {
    let Ok(egui_context) = world.query::<&mut EguiContext>().single_mut(world) else {
        return;
    };
    let mut egui_context = egui_context.clone();
    let ctx = egui_context.get_mut();
    // TODO: move egui options to one place
    // ctrl+plus / ctrl+minus / ctrl+zero is used for game viewport zooming in phichain. enabling this will cause ui glitch when using these hotkeys
    // even though there's no game preview in home page, we disable this for consistency
    ctx.options_mut(|options| options.zoom_with_keyboard = false);

    if world.query::<&LoadingProject>().single(world).is_ok() {
        egui::Modal::new(Id::new("home.loading")).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.add(egui::Label::new(t!("home.loading_project")).selectable(false));
            });
        });
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        let frame_time = ui.ctx().input(|i| i.time);
        if frame_time == 0.0 {
            return;
        }

        if world.get_resource::<OpenSettings>().is_some() {
            ui.horizontal(|ui| {
                if ui
                    .heading(egui_phosphor::regular::ARROW_LEFT)
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .on_hover_and_drag_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    world.remove_resource::<OpenSettings>();
                }
                ui.heading(t!("home.settings"));
            });
            ui.separator();
            settings_ui(ui, world);

            return;
        }

        ui.heading(format!("Phichain v{}", env!("CARGO_PKG_VERSION")));

        ui.separator();

        if world.contains_resource::<CreatingProject>() {
            let modal = egui::Modal::new(Id::new("home.create_project")).show(ctx, |ui| {
                ui.set_width(400.0);
                ui.heading(t!("home.create_project.label"));
                ui.separator();
                egui::Grid::new("create_project_grid")
                    .num_columns(2)
                    .spacing([20.0, 2.0])
                    .striped(true)
                    .show(ui, |ui| {
                        if ui.button(t!("home.create_project.select_music")).clicked() {
                            pick_file::<PickedMusic>(
                                world,
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
                            pick_file::<PickedIllustration>(
                                world,
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

                    pick_folder::<PickedCreateProject>(world, FileDialog::new());
                }
            });

            if modal.should_close() {
                world.remove_resource::<CreatingProject>();
            }
        }

        let languages = world.resource::<Languages>().0.clone();
        let editor_settings = world.resource::<Persistent<EditorSettings>>();

        let mut open_settings = false;

        let mut language_changed = false;
        let mut language = editor_settings.general.language.clone();

        let mut telemetry_changed = false;
        let mut telemetry = editor_settings.general.send_telemetry;

        ui.sides(
            |ui| {
                ui.horizontal(|ui| {
                    if ui.button(t!("home.open_project.load")).clicked() {
                        pick_folder::<PickedProject>(world, FileDialog::new());
                    }
                    if ui.button(t!("home.create_project.create")).clicked() {
                        world.insert_resource(CreatingProject);
                    }
                });
            },
            |ui| {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut telemetry, t!("home.telemetry")).changed() {
                        telemetry_changed = true;
                    }

                    if ui.button(t!("home.settings")).clicked() {
                        open_settings = true;
                    }

                    if language_combobox(ui, languages, &mut language) {
                        language_changed = true;
                    }

                    ui.label(
                        RichText::new(format!(
                            "{} {}",
                            egui_phosphor::regular::GLOBE,
                            t!("tab.settings.category.general.language.label")
                        ))
                        .color(Color32::LIGHT_BLUE),
                    );
                })
            },
        );

        if open_settings {
            world.insert_resource(OpenSettings);
        }

        if language_changed {
            let mut editor_settings = world.resource_mut::<Persistent<EditorSettings>>();
            editor_settings.general.language = language;
            let _ = editor_settings.persist();
        }

        if telemetry_changed {
            let mut editor_settings = world.resource_mut::<Persistent<EditorSettings>>();
            editor_settings.general.send_telemetry = telemetry;
            let _ = editor_settings.persist();
        }

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
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_and_drag_cursor(CursorIcon::PointingHand);
            }
        });
        ui.style_mut().interaction.selectable_labels = true;

        if let Some(index) = remove {
            recent_projects.remove(index);
        }

        if let Some(open) = open {
            world.write_message(LoadProjectEvent(open));
            world.remove_resource::<CreatingProject>();
        }
    });
}

fn load_project_observer(
    trigger: Trigger<PickedProject>,
    mut commands: Commands,
    mut events: MessageWriter<LoadProjectEvent>,
) {
    if let Some(ref root_dir) = trigger.event().0 {
        events.write(LoadProjectEvent(root_dir.to_path_buf()));
        commands.remove_resource::<CreatingProject>();
    }
}

fn handle_select_illustration_observer(
    trigger: Trigger<PickedIllustration>,
    mut form: ResMut<CreateProjectForm>,
) {
    form.illustration.clone_from(&trigger.event().0);
}

fn handle_select_music_observer(
    trigger: Trigger<PickedMusic>,
    mut form: ResMut<CreateProjectForm>,
) {
    form.music.clone_from(&trigger.event().0);
}

fn handle_create_project_observer(
    trigger: Trigger<PickedCreateProject>,
    mut commands: Commands,
    form: Res<CreateProjectForm>,
    mut load_project_events: MessageWriter<LoadProjectEvent>,
    mut toasts: ResMut<ToastsStorage>,
) {
    let Some(ref root_path) = trigger.event().0 else {
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
            load_project_events.write(LoadProjectEvent(root_path.clone()));
            commands.remove_resource::<CreatingProject>();
        }
        Err(error) => toasts.error(format!("{error:?}")),
    }
}
