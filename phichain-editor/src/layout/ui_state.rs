use crate::identifier::{Identifier, IntoIdentifier};
use crate::layout::dock_style::{paint_active_tab, show_dock_area};
use crate::tab::{EditorTab, TabRegistry};
use bevy::prelude::{Resource, World};
use egui_dock::{DockState, NodeIndex};
use serde::{Deserialize, Serialize};

struct TabViewer<'a> {
    world: &'a mut World,
    registry: &'a mut TabRegistry,
    active_tabs: Vec<Identifier>,
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct UiState {
    pub state: DockState<Identifier>,
}

impl Default for UiState {
    fn default() -> Self {
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
}

impl UiState {
    pub fn ui(&mut self, world: &mut World, registry: &mut TabRegistry, ctx: &mut egui::Context) {
        let mut tab_viewer = TabViewer {
            world,
            registry,
            active_tabs: self
                .state
                .iter_all_nodes()
                .filter_map(|(_, node)| node.get_leaf())
                .filter_map(|leaf| leaf.tabs.get(leaf.active.0).cloned())
                .collect(),
        };

        show_dock_area(ctx, &mut self.state, &mut tab_viewer);
    }
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Identifier;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        t!(format!("tab.{tab}.title").as_str()).into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        self.registry.tab_ui(ui, self.world, tab);
    }

    fn on_tab_button(&mut self, tab: &mut Self::Tab, response: &egui::Response) {
        if self.active_tabs.contains(tab) {
            paint_active_tab(response);
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        self.allowed_in_windows(tab)
    }

    fn allowed_in_windows(&self, tab: &mut Self::Tab) -> bool {
        tab.to_string() != EditorTab::Game.into_identifier().to_string()
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        *tab != EditorTab::Game.into_identifier()
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
