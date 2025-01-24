mod action_panel;
pub mod bpm_list;
pub mod chart_basic_setting;
pub mod game;
pub mod inspector;
pub mod line_list;
pub mod quick_action;
pub mod settings;
pub mod timeline;
pub mod timeline_setting;

use crate::identifier::{Identifier, IntoIdentifier};
use crate::tab::action_panel::ActionPanelPlugin;
use crate::tab::bpm_list::bpm_list_tab;
use crate::tab::chart_basic_setting::chart_basic_setting_tab;
use crate::tab::game::game_tab;
use crate::tab::inspector::inspector_ui_system;
use crate::tab::line_list::line_list_tab;
use crate::tab::settings::settings_tab;
use crate::tab::timeline::timeline_tab;
use crate::tab::timeline_setting::timeline_setting_tab;
use bevy::{prelude::*, utils::HashMap};
use egui::Ui;

#[allow(dead_code)]
pub fn empty_tab(In(_): In<Ui>) {}

pub struct RegisteredTab {
    system: Box<dyn System<In = Ui, Out = ()>>,
}

impl RegisteredTab {
    pub fn run(&mut self, world: &mut World, ui: &mut Ui) {
        let child = ui.child_ui(ui.max_rect(), *ui.layout());
        self.system.run(child, world);
    }
}

#[derive(Resource, Deref, Default)]
pub struct TabRegistry(HashMap<Identifier, RegisteredTab>);

impl TabRegistry {
    pub fn tab_ui(&mut self, ui: &mut Ui, world: &mut World, tab: &Identifier) {
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

impl IntoIdentifier for EditorTab {
    fn into_identifier(self) -> Identifier {
        match self {
            EditorTab::Game => "game".into(),
            EditorTab::Timeline => "timeline".into(),
            EditorTab::Inspector => "inspector".into(),
            EditorTab::TimelineSetting => "timeline_setting".into(),
            EditorTab::ChartBasicSetting => "chart_basic_setting".into(),
            EditorTab::LineList => "line_list".into(),
            EditorTab::BpmList => "bpm_list".into(),
            EditorTab::Settings => "settings".into(),
        }
    }
}

pub struct TabPlugin;

impl Plugin for TabPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TabRegistry>()
            .register_tab(EditorTab::Timeline, timeline_tab)
            .register_tab(EditorTab::Game, game_tab)
            .register_tab(EditorTab::Inspector, inspector_ui_system)
            .register_tab(EditorTab::TimelineSetting, timeline_setting_tab)
            .register_tab(EditorTab::ChartBasicSetting, chart_basic_setting_tab)
            .register_tab(EditorTab::BpmList, bpm_list_tab)
            .register_tab(EditorTab::LineList, line_list_tab)
            .register_tab(EditorTab::Settings, settings_tab)
            .add_plugins(ActionPanelPlugin);
    }
}

pub trait TabRegistrationExt {
    fn register_tab<M1>(
        &mut self,
        id: impl IntoIdentifier,
        system: impl IntoSystem<Ui, (), M1>,
    ) -> &mut Self;
}

impl TabRegistrationExt for App {
    fn register_tab<M1>(
        &mut self,
        id: impl IntoIdentifier,
        system: impl IntoSystem<Ui, (), M1>,
    ) -> &mut Self {
        self.world
            .resource_scope(|world, mut registry: Mut<TabRegistry>| {
                registry.0.insert(
                    id.into_identifier(),
                    RegisteredTab {
                        system: Box::new({
                            let mut sys = IntoSystem::into_system(system);
                            sys.initialize(world);
                            sys
                        }),
                    },
                )
            });
        self
    }
}
