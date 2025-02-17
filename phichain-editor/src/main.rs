#[macro_use]
extern crate rust_i18n;

mod action;
mod audio;
mod cli;
mod constants;
mod editing;
mod events;
mod export;
mod exporter;
mod file;
mod hit_sound;
mod home;
mod hotkey;
mod identifier;
mod misc;
mod notification;
mod project;
mod recent_projects;
mod schedule;
mod screenshot;
mod selection;
mod serialization;
mod settings;
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
use crate::cli::{Args, CliPlugin};
use crate::editing::history::EditorHistory;
use crate::editing::EditingPlugin;
use crate::events::EventPlugin;
use crate::export::ExportPlugin;
use crate::file::FilePickingPlugin;
use crate::hit_sound::HitSoundPlugin;
use crate::home::HomePlugin;
use crate::hotkey::HotkeyPlugin;
use crate::identifier::{Identifier, IntoIdentifier};
use crate::misc::{MiscPlugin, WorkingDirectory};
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
use crate::tab::{EditorTab, TabRegistry};
use crate::telemetry::TelemetryPlugin;
use crate::timeline::TimelinePlugin;
use crate::timing::TimingPlugin;
use crate::translation::TranslationPlugin;
use crate::ui::UiPlugin;
use crate::zoom::ZoomPlugin;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::log::tracing_subscriber::Layer;
use bevy::log::{tracing_subscriber, BoxedLayer, LogPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::WgpuFeatures;
use bevy::render::settings::WgpuSettings;
use bevy::render::RenderPlugin;
use bevy_egui::egui::{Color32, Frame};
use bevy_egui::{EguiContext, EguiPlugin};
use bevy_mod_reqwest::ReqwestPlugin;
use bevy_persistent::Persistent;
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use phichain_assets::AssetsPlugin;
use phichain_chart::event::LineEvent;
use phichain_chart::note::Note;
use phichain_game::{GamePlugin, GameSet};
use rust_i18n::set_locale;
use std::env;
use std::sync::Arc;

i18n!("lang", fallback = "en_us");

fn main() {
    let mut wgpu_settings = WgpuSettings::default();
    wgpu_settings
        .features
        .set(WgpuFeatures::VERTEX_WRITABLE_STORAGE, true);

    phichain_assets::setup_assets();

    App::new()
        .configure_sets(Update, GameSet.run_if(project_loaded()))
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
                    render_creation: wgpu_settings.into(),
                    synchronous_pipeline_compilation: false,
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
        .add_plugins(GamePlugin)
        .add_plugins(ActionPlugin)
        .add_plugins(ScreenshotPlugin)
        .add_plugins(TimingPlugin)
        .add_plugins(AudioPlugin)
        .add_plugins(EditorSettingsPlugin)
        .add_plugins(HitSoundPlugin)
        .add_plugins(GameTabPlugin)
        .add_plugins(TimelinePlugin)
        .add_plugins(EguiPlugin)
        .add_plugins(ProjectPlugin)
        .add_plugins(ExportPlugin)
        .add_plugins(selection::SelectionPlugin)
        .add_plugins(TabPlugin)
        .add_plugins(EditingPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(AssetsPlugin)
        .add_plugins(NotificationPlugin)
        .add_plugins(FilePickingPlugin)
        .add_plugins(EventPlugin)
        .add_plugins(ZoomPlugin)
        .add_systems(Startup, setup_egui_image_loader_system)
        .add_systems(Startup, setup_egui_font_system)
        .add_systems(Startup, setup_system)
        .add_systems(Update, ui_system.run_if(project_loaded()))
        .add_systems(
            Startup,
            (apply_args_config_system, apply_editor_settings_system),
        )
        .run();
}

/// Hold the [`tracing_appender`] guard
#[derive(Resource)]
#[allow(dead_code)]
struct LogGuard(tracing_appender::non_blocking::WorkerGuard);

fn custom_layer(app: &mut App) -> Option<BoxedLayer> {
    let path = app.world().resource::<WorkingDirectory>().log().ok()?;

    let appender = tracing_appender::rolling::never(path, "phichain.log");

    let (non_blocking, guard) = tracing_appender::non_blocking(appender);

    app.insert_resource(LogGuard(guard));

    Some(Box::new(vec![tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .boxed()]))
}

fn apply_editor_settings_system(settings: Res<Persistent<EditorSettings>>) {
    set_locale(settings.general.language.as_str());
}

/// Apply configurations from the command line args
fn apply_args_config_system(args: Res<Args>, mut events: EventWriter<LoadProjectEvent>) {
    // load chart if specified
    if let Some(path) = &args.project {
        events.send(LoadProjectEvent(path.into()));
    }
}

fn setup_egui_image_loader_system(mut contexts: bevy_egui::EguiContexts) {
    egui_extras::install_image_loaders(contexts.ctx_mut());
}

fn setup_egui_font_system(mut contexts: bevy_egui::EguiContexts) {
    let ctx = contexts.ctx_mut();

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
}

struct TabViewer<'a> {
    world: &'a mut World,
    registry: &'a mut TabRegistry,
}

#[derive(Resource)]
struct UiState {
    state: DockState<Identifier>,
}

impl UiState {
    fn new() -> Self {
        let mut state = DockState::new(vec![EditorTab::Game.into_identifier()]);
        let tree = state.main_surface_mut();
        let [game, timeline] = tree.split_left(
            NodeIndex::root(),
            2.0 / 3.0,
            vec![
                EditorTab::Timeline.into_identifier(),
                EditorTab::Settings.into_identifier(),
            ],
        );

        let [_line_list, _timeline] = tree.split_left(
            timeline,
            1.0 / 4.0,
            vec![EditorTab::LineList.into_identifier()],
        );

        let [_, inspector] = tree.split_below(
            game,
            2.0 / 5.0,
            vec![EditorTab::Inspector.into_identifier()],
        );
        tree.split_right(
            inspector,
            1.0 / 2.0,
            vec![EditorTab::TimelineSetting.into_identifier()],
        );

        Self { state }
    }

    fn ui(&mut self, world: &mut World, registry: &mut TabRegistry, ctx: &mut egui::Context) {
        let mut tab_viewer = TabViewer { world, registry };

        DockArea::new(&mut self.state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);
    }
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Identifier;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        t!(format!("tab.{}.title", tab).as_str()).into()
    }
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        self.registry.tab_ui(ui, self.world, tab);
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        self.allowed_in_windows(tab)
    }

    fn allowed_in_windows(&self, tab: &mut Self::Tab) -> bool {
        tab.to_string() != EditorTab::Game.into_identifier().to_string()
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        *tab != EditorTab::Game.into_identifier() && *tab != EditorTab::Timeline.into_identifier()
    }

    fn scroll_bars(&self, tab: &Self::Tab) -> [bool; 2] {
        if *tab == EditorTab::Game.into_identifier()
            || *tab == EditorTab::Timeline.into_identifier()
        {
            [false, false]
        } else {
            [true, true]
        }
    }
}

fn ui_system(world: &mut World) {
    let Ok(egui_context) = world.query::<&mut EguiContext>().get_single_mut(world) else {
        return;
    };
    let mut egui_context = egui_context.clone();
    let ctx = egui_context.get_mut();
    // TODO: move egui options to one place
    // ctrl+plus / ctrl+minus / ctrl+zero is used for game viewport zooming in phichain. enabling this will cause ui glitch when using these hotkeys
    ctx.options_mut(|options| options.zoom_with_keyboard = false);

    let diagnostics = world.resource::<DiagnosticsStore>();
    let mut fps = 0.0;
    if let Some(value) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
    {
        fps = value;
    }

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
                                .selectable_label(opened, t!(format!("tab.{}.title", tab).as_str()))
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

            ui.label(format!("FPS: {:.2}", fps));

            ui.label(format!("Notes: {}", notes));
            ui.label(format!("Events: {}", events));

            ui.label(format!("Selected Notes: {}", selected_notes));
            ui.label(format!("Selected Events: {}", selected_events));

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
    commands.spawn((Camera2d, GameCamera));
}
