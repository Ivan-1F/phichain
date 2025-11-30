#[macro_use]
extern crate rust_i18n;

mod action;
mod audio;
mod autosave;
mod cli;
mod constants;
mod editing;
mod events;
mod export;
mod file;
mod fps;
mod hit_sound;
mod home;
mod hotkey;
mod identifier;
mod ime;
mod l10n;
mod layout;
mod logging;
mod metronome;
mod misc;
mod notification;
mod project;
mod recent_projects;
mod schedule;
mod screenshot;
mod selection;
mod settings;
mod spectrogram;
mod tab;
mod telemetry;
mod timeline;
mod timing;
mod translation;
mod ui;
mod utils;
mod zoom;

use crate::action::{ActionPlugin, ActionRegistry};
use crate::audio::AudioPlugin;
use crate::autosave::AutoSavePlugin;
use crate::cli::{Args, CliPlugin};
use crate::editing::history::EditorHistory;
use crate::editing::EditingPlugin;
use crate::events::EventPlugin;
use crate::export::ExportPlugin;
use crate::file::FilePickingPlugin;
use crate::fps::{FpsDisplay, FpsPlugin};
use crate::hit_sound::HitSoundPlugin;
use crate::home::HomePlugin;
use crate::hotkey::HotkeyPlugin;
use crate::ime::ImeCompatPlugin;
use crate::layout::ui_state::UiState;
use crate::layout::{layout_menu, LayoutPlugin};
use crate::logging::custom_layer;
use crate::metronome::MetronomePlugin;
use crate::misc::MiscPlugin;
use crate::notification::NotificationPlugin;
use crate::project::project_loaded;
use crate::project::LoadProjectEvent;
use crate::project::ProjectPlugin;
use crate::recent_projects::RecentProjectsPlugin;
use crate::schedule::EditorSet;
use crate::screenshot::ScreenshotPlugin;
use crate::selection::Selected;
use crate::settings::{EditorSettings, EditorSettingsPlugin};
use crate::tab::game::GameCamera;
use crate::tab::game::GameTabPlugin;
use crate::tab::quick_action::quick_action;
use crate::tab::TabPlugin;
use crate::tab::TabRegistry;
use crate::telemetry::TelemetryPlugin;
use crate::timeline::TimelinePlugin;
use crate::timing::TimingPlugin;
use crate::translation::TranslationPlugin;
use crate::ui::UiPlugin;
use crate::zoom::ZoomPlugin;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::render::RenderPlugin;
use bevy_egui::egui::{Color32, Frame};
use bevy_egui::{
    EguiContext, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext,
};
use bevy_mod_reqwest::ReqwestPlugin;
use bevy_persistent::Persistent;
use egui::RichText;
use phichain_assets::AssetsPlugin;
use phichain_chart::event::LineEvent;
use phichain_chart::note::Note;
use phichain_game::{GamePlugin, GameSet};
use rust_i18n::set_locale;
use std::env;
use std::sync::Arc;

i18n!("lang", fallback = "en_us");

fn main() {
    if let Err(err) = logging::roll_latest() {
        error!("Failed to roll latest.log: {}", err);
    }

    phichain_assets::setup_assets();

    let mut app = App::new();

    app.configure_sets(Update, GameSet.run_if(project_loaded()))
        .configure_sets(
            Update,
            (EditorSet::Edit, EditorSet::Update)
                .chain()
                .before(GameSet)
                .run_if(project_loaded()),
        )
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(UiState::new())
        .add_plugins(ReqwestPlugin::default())
        .add_plugins(HotkeyPlugin)
        .add_plugins(CliPlugin)
        .add_plugins(MiscPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(TranslationPlugin)
        .add_plugins(RecentProjectsPlugin)
        .add_plugins(HomePlugin)
        .add_plugins(TelemetryPlugin)
        .add_plugins(
            DefaultPlugins
                .set(RenderPlugin {
                    synchronous_pipeline_compilation: false,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Phichain".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    custom_layer,
                    ..default()
                }),
        )
        .add_plugins(ImeCompatPlugin)
        .add_plugins(GamePlugin)
        .add_plugins(ActionPlugin)
        .add_plugins(AutoSavePlugin)
        .add_plugins(ScreenshotPlugin)
        .add_plugins(TimingPlugin)
        .add_plugins(AudioPlugin)
        .add_plugins(EditorSettingsPlugin)
        .add_plugins(HitSoundPlugin)
        .add_plugins(MetronomePlugin)
        .add_plugins(GameTabPlugin)
        .add_plugins(TimelinePlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(ProjectPlugin)
        .add_plugins(ExportPlugin)
        .add_plugins(selection::SelectionPlugin)
        .add_plugins(TabPlugin)
        .add_plugins(EditingPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(AssetsPlugin)
        .add_plugins(NotificationPlugin)
        .add_plugins(FilePickingPlugin)
        .add_plugins(EventPlugin)
        .add_plugins(ZoomPlugin)
        .add_plugins(FpsPlugin)
        .add_plugins(LayoutPlugin)
        .add_systems(Startup, setup_system)
        .add_systems(Startup, setup_egui_system)
        .add_systems(
            EguiPrimaryContextPass,
            setup_egui_image_loader_system.run_if(run_once),
        )
        .add_systems(
            EguiPrimaryContextPass,
            setup_egui_font_system.run_if(run_once),
        )
        .add_systems(EguiPrimaryContextPass, ui_system.run_if(project_loaded()))
        .add_systems(
            Startup,
            (apply_args_config_system, apply_editor_settings_system),
        )
        .add_systems(Update, update_ui_scale_changes_system);

    // #[cfg(debug_assertions)]
    // app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());

    app.run();
}

fn apply_editor_settings_system(settings: Res<Persistent<EditorSettings>>) {
    set_locale(settings.general.language.as_str());
}

/// Apply configurations from the command line args
fn apply_args_config_system(args: Res<Args>, mut events: EventWriter<LoadProjectEvent>) {
    // load chart if specified
    if let Some(path) = &args.project {
        events.write(LoadProjectEvent(path.into()));
    }
}

fn update_ui_scale_changes_system(
    settings: Res<Persistent<EditorSettings>>,
    mut windows: Query<&mut Window>,
) {
    if settings.is_changed() {
        if let Ok(mut window) = windows.single_mut() {
            // Preserve current window dimensions and only update scale factor
            let current_resolution = &mut window.resolution;
            current_resolution.set_scale_factor_override(Some(settings.general.ui_scale));

            // Force window to apply the new scale by triggering a minimal resize using logical dimensions
            let current_width = current_resolution.width();
            let current_height = current_resolution.height();
            current_resolution.set(current_width, current_height);
        }
    }
}

fn setup_egui_image_loader_system(mut contexts: bevy_egui::EguiContexts) -> Result {
    egui_extras::install_image_loaders(contexts.ctx_mut()?);

    Ok(())
}

fn setup_egui_font_system(mut contexts: bevy_egui::EguiContexts) -> Result {
    let ctx = contexts.ctx_mut()?;

    let font_file = utils::assets::get_base_path()
        .join("assets/font/MiSans-Regular.ttf")
        .to_str()
        .unwrap()
        .to_string();
    let font_name = "MiSans-Regular".to_string();
    let font_file_bytes = std::fs::read(font_file).expect("Failed to open font file");

    let font_data = egui::FontData::from_owned(font_file_bytes);
    let mut font_def = egui::FontDefinitions::default();
    font_def
        .font_data
        .insert(font_name.to_string(), Arc::new(font_data));

    let font_family: egui::FontFamily = egui::FontFamily::Proportional;
    font_def
        .families
        .get_mut(&font_family)
        .expect("Failed to setup font")
        .insert(0, font_name);

    egui_phosphor::add_to_fonts(&mut font_def, egui_phosphor::Variant::Regular);

    ctx.set_fonts(font_def);

    Ok(())
}

fn ui_system(world: &mut World) {
    let Ok(egui_context) = world.query::<&mut EguiContext>().single_mut(world) else {
        return;
    };
    let mut egui_context = egui_context.clone();
    let ctx = egui_context.get_mut();
    // TODO: move egui options to one place
    // ctrl+plus / ctrl+minus / ctrl+zero is used for game viewport zooming in phichain. enabling this will cause ui glitch when using these hotkeys
    ctx.options_mut(|options| options.zoom_with_keyboard = false);

    let fps_display = world.resource::<FpsDisplay>();
    let fps = fps_display.displayed_fps;

    egui::TopBottomPanel::top("phichain.MenuBar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button(t!("menu_bar.file.title"), |ui| {
                if ui.button(t!("menu_bar.file.save")).clicked() {
                    world.resource_scope(|world, mut registry: Mut<ActionRegistry>| {
                        registry.run_action(world, "phichain.save_project");
                    });
                    ui.close_menu();
                }
                if ui.button(t!("menu_bar.file.close")).clicked() {
                    world.resource_scope(|world, mut registry: Mut<ActionRegistry>| {
                        registry.run_action(world, "phichain.close_project");
                    });
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t!("menu_bar.file.quit")).clicked() {
                    std::process::exit(0);
                }
            });
            ui.menu_button(t!("menu_bar.tabs.title"), |ui| {
                world.resource_scope(|world, mut ui_state: Mut<UiState>| {
                    world.resource_scope(|_, registry: Mut<TabRegistry>| {
                        for (tab, _) in registry.iter() {
                            let opened = ui_state
                                .state
                                .iter_all_tabs()
                                .map(|x| x.1)
                                .collect::<Vec<_>>()
                                .contains(&tab);
                            if ui
                                .selectable_label(opened, t!(format!("tab.{tab}.title").as_str()))
                                .clicked()
                            {
                                if opened {
                                    if let Some(node) = ui_state.state.find_tab(tab) {
                                        ui_state.state.remove_tab(node);
                                    }
                                    ui.close_menu();
                                } else {
                                    ui_state.state.add_window(vec![tab.clone()]);
                                    ui.close_menu();
                                }
                            }
                        }
                    });
                });
            });

            ui.menu_button(t!("menu_bar.export.title"), |ui| {
                if ui.button(t!("menu_bar.export.as_official")).clicked() {
                    // TODO: make menu bar powered by actions
                    world.resource_scope(|world, mut actions: Mut<ActionRegistry>| {
                        actions.run_action(world, "phichain.export_as_official");
                    });
                    ui.close_menu();
                }
            });

            layout_menu(ui, world);
        });

        ui.add(
            egui::Separator::default()
                .spacing(1.0)
                // fill the left and right gap
                .grow(20.0),
        );

        quick_action(ui, world);

        ui.add_space(1.0);
    });

    let notes: Vec<_> = world.query::<&Note>().iter(world).collect();
    let notes = notes.len();
    let events: Vec<_> = world.query::<&LineEvent>().iter(world).collect();
    let events = events.len();

    let selected_notes: Vec<_> = world
        .query_filtered::<&Note, With<Selected>>()
        .iter(world)
        .collect();
    let selected_notes = selected_notes.len();
    let selected_events: Vec<_> = world
        .query_filtered::<&LineEvent, With<Selected>>()
        .iter(world)
        .collect();
    let selected_events = selected_events.len();

    egui::TopBottomPanel::bottom("phichain.StatusBar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(format!("Phichain v{}", env!("CARGO_PKG_VERSION")));

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("FPS: ");
                ui.label(RichText::new(format!("{fps:.1}")).monospace());
            });

            ui.label(format!("Notes: {notes}"));
            ui.label(format!("Events: {events}"));

            ui.label(format!("Selected Notes: {selected_notes}"));
            ui.label(format!("Selected Events: {selected_events}"));

            world.resource_scope(|_world: &mut World, history: Mut<EditorHistory>| {
                if !history.0.is_saved() {
                    ui.label("*");
                }
            });
        });
    });

    egui::CentralPanel::default()
        .frame(Frame {
            fill: Color32::TRANSPARENT,
            ..default()
        })
        .show(ctx, |_ui| {
            world.resource_scope(|world: &mut World, mut registry: Mut<TabRegistry>| {
                world.resource_scope(|world: &mut World, mut ui_state: Mut<UiState>| {
                    ui_state.ui(world, &mut registry, &mut ctx.clone());
                });
            });
        });
}

fn setup_system(mut commands: Commands) {
    // this is the camera which renders the game content, including the game ui. its viewport will be limited to game_tab's clip_rect
    commands.spawn((Camera2d, GameCamera, IsDefaultUiCamera));
}

fn setup_egui_system(mut commands: Commands, mut egui_global_settings: ResMut<EguiGlobalSettings>) {
    egui_global_settings.auto_create_primary_context = false;

    // this is the camera which renders egui. its viewport will be the entire screen
    commands.spawn((
        // The `PrimaryEguiContext` component requires everything needed to render a primary context.
        PrimaryEguiContext,
        Camera2d,
        // Setting RenderLayers to none makes sure we won't render anything apart from the UI.
        RenderLayers::none(),
        Camera {
            order: 1,
            ..default()
        },
    ));
}
