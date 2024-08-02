pub mod bpm_list;
pub mod chart_basic_setting;
pub mod game;
pub mod inspector;
pub mod line_list;
pub mod quick_action;
pub mod settings;
pub mod timeline;
pub mod timeline_setting;

use crate::tab::bpm_list::bpm_list_tab;
use crate::tab::chart_basic_setting::chart_basic_setting_tab;
use crate::tab::inspector::inspector_ui_system;
use crate::tab::line_list::line_list_tab;
use crate::tab::settings::settings_tab;
use crate::tab::timeline::timeline_tab;
use crate::tab::timeline_setting::timeline_setting_tab;
use bevy::{prelude::*, utils::HashMap};
use egui::Ui;

pub fn empty_tab(In(_ui): In<&mut Ui>) {}

pub struct RegisteredTab {
    system: Box<dyn System<In = &'static mut Ui, Out = ()>>,
    tab_title: &'static str,
}

impl RegisteredTab {
    pub fn run(&mut self, world: &mut World, ui: &mut Ui) {
        unsafe {
            self.system.run(&mut *(ui as *mut Ui), world);
        }
    }

    pub fn title(&self) -> &'static str {
        self.tab_title
    }
}

#[derive(Resource, Deref, Default)]
pub struct TabRegistry(HashMap<EditorTab, RegisteredTab>);

impl TabRegistry {
    pub fn tab_ui(&mut self, ui: &mut Ui, world: &mut World, tab: &EditorTab) {
        if let Some(tab) = self.0.get_mut(tab) {
            tab.run(world, ui);
        } else {
            ui.colored_label(egui::Color32::RED, "Tab does not exist.".to_string());
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum EditorTab {
    Game,
    Timeline,
    Inspector,
    TimelineSetting,
    ChartBasicSetting,
    LineList,
    BpmList,
    Settings,
}

pub struct TabPlugin;

impl Plugin for TabPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TabRegistry>()
            .register_tab(EditorTab::Timeline, "tab.timeline.title", timeline_tab)
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
            .register_tab(
                EditorTab::ChartBasicSetting,
                "tab.chart_basic_setting.title",
                chart_basic_setting_tab,
            )
            .register_tab(EditorTab::BpmList, "tab.bpm_list.title", bpm_list_tab)
            .register_tab(EditorTab::LineList, "tab.line_list.title", line_list_tab)
            .register_tab(EditorTab::Settings, "tab.settings.title", settings_tab);
    }
}

pub trait TabRegistrationExt {
    fn register_tab<M1>(
        &mut self,
        id: impl Into<EditorTab>,
        name: &'static str,
        system: impl IntoSystem<&'static mut Ui, (), M1>,
    ) -> &mut Self;
}

impl TabRegistrationExt for App {
    fn register_tab<M1>(
        &mut self,
        id: impl Into<EditorTab>,
        name: &'static str,
        system: impl IntoSystem<&'static mut Ui, (), M1>,
    ) -> &mut Self {
        self.world
            .resource_scope(|world, mut registry: Mut<TabRegistry>| {
                registry.0.insert(
                    id.into(),
                    RegisteredTab {
                        system: Box::new({
                            let mut sys = IntoSystem::into_system(system);
                            sys.initialize(world);
                            sys
                        }),
                        tab_title: name,
                    },
                )
            });
        self
    }
}
