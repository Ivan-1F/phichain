mod assets;
mod audio;
mod chart;
mod constants;
mod exporter;
mod hit_sound;
mod home;
mod loader;
mod misc;
mod project;
mod score;
mod selection;
mod serialzation;
mod tab;
mod timing;
mod translation;

use crate::assets::AssetsPlugin;
use crate::audio::AudioPlugin;
use crate::chart::event::LineEvent;
use crate::chart::note::Note;
use crate::exporter::phichain::PhiChainExporter;
use crate::exporter::Exporter;
use crate::home::HomePlugin;
use crate::misc::MiscPlugin;
use crate::misc::WorkingDirectory;
use crate::project::project_loaded;
use crate::project::ProjectPlugin;
use crate::score::ScorePlugin;
use crate::tab::game::GameCamera;
use crate::tab::game::GameTabPlugin;
use crate::tab::game::GameViewport;
use crate::tab::inspector::inspector_ui_system;
use crate::tab::timeline::timeline_ui_system;
use crate::tab::timeline::{TimelineTabPlugin, TimelineViewport};
use crate::tab::timeline_setting::timeline_setting_tab;
use crate::tab::TabPlugin;
use crate::tab::{empty_tab, EditorTab, TabRegistrationExt, TabRegistry};
use crate::timing::TimingPlugin;
use crate::translation::{TranslationPlugin, Translator};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy_egui::egui::{Color32, Frame};
use bevy_egui::{EguiContext, EguiPlugin};
use bevy_mod_picking::prelude::*;
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use project::LoadProjectEvent;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(UiState::new())
        .add_plugins(HomePlugin)
        .add_plugins(DefaultPlugins)
        .add_plugins(TimingPlugin)
        .add_plugins(AudioPlugin)
        .add_plugins(GameTabPlugin)
        .add_plugins(ScorePlugin)
        .add_plugins(TimelineTabPlugin)
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(ProjectPlugin)
        .add_plugins(crate::selection::SelectionPlugin)
        .add_plugins(MiscPlugin)
        .add_plugins(TabPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(AssetsPlugin)
        .add_plugins(TranslationPlugin)
        .add_systems(Startup, setup_egui_image_loader_system)
        .add_systems(Startup, setup_egui_font_system)
        .add_systems(Startup, setup_plugin)
        .add_systems(Update, ui_system.run_if(project_loaded()))
        .add_systems(Update, |world: &mut World| {
            // Debug
            let event = world.resource::<ButtonInput<KeyCode>>();
            if event.just_pressed(KeyCode::KeyE) {
                if let Ok(chart) = PhiChainExporter::export(world) {
                    let _ = std::fs::write("Chart.json", chart);
                }
            }
        })
        .add_systems(Startup, load_project_from_args)
        .register_tab(
            EditorTab::Timeline,
            "tab.timeline.title",
            timeline_ui_system,
        )
        .register_tab(EditorTab::Game, "tab.game.title", empty_tab)
        .register_tab(
            EditorTab::Inspector,
            "tab.inspector.title",
            inspector_ui_system,
        )
        .register_tab(
            EditorTab::TimelineSetting,
            "tab.timeline_setting.title",
            timeline_setting_tab,
        )
        .run();
}

fn load_project_from_args(mut events: EventWriter<LoadProjectEvent>) {
    if let Some(path) = std::env::args().nth(1) {
        events.send(LoadProjectEvent(path.into()));
    }
}

fn setup_egui_image_loader_system(mut contexts: bevy_egui::EguiContexts) {
    egui_extras::install_image_loaders(contexts.ctx_mut());
}

fn setup_egui_font_system(
    mut contexts: bevy_egui::EguiContexts,
    working_directory: Res<WorkingDirectory>,
) {
    let ctx = contexts.ctx_mut();

    let font_file = working_directory
        .0
        .join("assets/font/MiSans-Regular.ttf")
        .to_str()
        .unwrap()
        .to_string();
    let font_name = "MiSans-Regular".to_string();
    let font_file_bytes = std::fs::read(font_file).expect("Failed to open font file");

    let font_data = egui::FontData::from_owned(font_file_bytes);
    let mut font_def = egui::FontDefinitions::default();
    font_def.font_data.insert(font_name.to_string(), font_data);

    let font_family: egui::FontFamily = egui::FontFamily::Proportional;
    font_def
        .families
        .get_mut(&font_family)
        .expect("Failed to setuo font")
        .insert(0, font_name);

    ctx.set_fonts(font_def);
}

struct TabViewer<'a> {
    world: &'a mut World,
    registry: &'a mut TabRegistry,
}

#[derive(Resource)]
struct UiState {
    state: DockState<EditorTab>,
}

impl UiState {
    fn new() -> Self {
        let mut state = DockState::new(vec![EditorTab::Game]);
        let tree = state.main_surface_mut();
        let [game, _timeline] =
            tree.split_left(NodeIndex::root(), 2.0 / 3.0, vec![EditorTab::Timeline]);

        let [_, inspector] = tree.split_below(game, 2.0 / 5.0, vec![EditorTab::Inspector]);
        tree.split_right(inspector, 1.0 / 2.0, vec![EditorTab::TimelineSetting]);

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
    type Tab = EditorTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        let mut system_state: SystemState<Translator> = SystemState::new(self.world);
        let translator = system_state.get(self.world);

        self.registry
            .get(tab)
            .map(|t| translator.tr(t.title()))
            .unwrap_or("Unknown".to_string())
            .into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        self.registry.tab_ui(ui, self.world, tab);
        match tab {
            EditorTab::Game => {
                let mut game_viewport = self.world.resource_mut::<GameViewport>();
                let clip_rect = ui.clip_rect();
                game_viewport.0 = Rect::from_corners(
                    Vec2 {
                        x: clip_rect.min.x,
                        y: clip_rect.min.y,
                    },
                    Vec2 {
                        x: clip_rect.max.x,
                        y: clip_rect.max.y,
                    },
                );
            }
            EditorTab::Timeline => {
                let mut timeline_viewport = self.world.resource_mut::<TimelineViewport>();
                let clip_rect = ui.clip_rect();
                timeline_viewport.0 = Rect::from_corners(
                    Vec2 {
                        x: clip_rect.min.x,
                        y: clip_rect.min.y,
                    },
                    Vec2 {
                        x: clip_rect.max.x,
                        y: clip_rect.max.y,
                    },
                );
            }
            _ => {}
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        self.allowed_in_windows(tab)
    }

    fn allowed_in_windows(&self, tab: &mut Self::Tab) -> bool {
        !matches!(tab, EditorTab::Game | EditorTab::Timeline)
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, EditorTab::Game | EditorTab::Timeline)
    }
}

fn ui_system(world: &mut World) {
    let egui_context = world.query::<&mut EguiContext>().single_mut(world);
    let mut egui_context = egui_context.clone();
    let ctx = egui_context.get_mut();

    let diagnostics = world.resource::<DiagnosticsStore>();
    let mut fps = 0.0;
    if let Some(value) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
    {
        fps = value;
    }

    let mut system_state: SystemState<Translator> = SystemState::new(world);
    let translator = system_state.get(world);
    egui::TopBottomPanel::top("phichain.MenuBar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button(translator.tr("menu_bar.file"), |ui| {
                if ui.button(translator.tr("menu_bar.file.quit")).clicked() {
                    std::process::exit(0);
                }
            });
        });
    });

    let notes: Vec<_> = world.query::<&Note>().iter(world).collect();
    let notes = notes.len();
    let events: Vec<_> = world.query::<&LineEvent>().iter(world).collect();
    let events = events.len();

    egui::TopBottomPanel::bottom("phichain.StatusBar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(format!("PhiChain v{}", env!("CARGO_PKG_VERSION")));

            ui.label(format!("FPS: {:.2}", fps));

            ui.label(format!("Notes: {}", notes));
            ui.label(format!("Events: {}", events));
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

fn setup_plugin(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 0,
                ..default()
            },
            ..default()
        },
        GameCamera,
    ));
}
