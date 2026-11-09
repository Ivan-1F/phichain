use crate::capture::locate::{by_name, by_role, panel};
use crate::capture::story::{Anchor, Story};
use crate::tab::EditorTab;
use bevy_egui::egui::accesskit::Role;
use phichain_chart::project::ProjectMeta;

pub fn quick_start() -> Story {
    let mut story = Story::new(ProjectMeta {
        name: "Quick Start".into(),
        ..Default::default()
    });

    story.image("editor-overview").anchors(
        [
            EditorTab::Game,
            EditorTab::Timeline,
            EditorTab::LineList,
            EditorTab::Inspector,
            EditorTab::TimelineSetting,
        ]
        .map(Anchor::panel),
    );

    story.click(by_role(Role::Button).name("menu_bar.tabs.title"));
    story.click(by_role(Role::Button).name("tab.bpm_list.title"));
    story
        .image("bpm-list")
        .anchor(Anchor::new("bpm_list.panel", panel(EditorTab::BpmList)))
        .anchor(Anchor::new(
            "bpm_list.point_0.bpm",
            panel(EditorTab::BpmList)
                .find(by_name("tab.bpm_list.point.bpm"))
                .find(by_role(Role::SpinButton)),
        ));

    story
}
