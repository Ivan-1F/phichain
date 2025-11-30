use crate::identifier::{Identifier, IntoIdentifier};
use crate::tab::{EditorTab, TabRegistry};
use bevy::prelude::{Resource, World};
use egui_dock::{DockArea, DockState, NodeIndex, Style};

struct TabViewer<'a> {
    world: &'a mut World,
    registry: &'a mut TabRegistry,
}

#[derive(Resource)]
pub struct UiState {
    pub state: DockState<Identifier>,
}

impl UiState {
    pub fn new() -> Self {
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

    pub fn ui(&mut self, world: &mut World, registry: &mut TabRegistry, ctx: &mut egui::Context) {
        let mut tab_viewer = TabViewer { world, registry };

        DockArea::new(&mut self.state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);
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
