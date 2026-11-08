use crate::capture::story::{Anchor, Story};
use crate::tab::EditorTab;
use phichain_chart::project::ProjectMeta;

pub fn quick_start() -> Story {
    let mut story = Story::new(ProjectMeta {
        name: "Quick Start".into(),
        ..Default::default()
    });

    story.image(
        "editor-overview",
        [
            EditorTab::Game,
            EditorTab::Timeline,
            EditorTab::LineList,
            EditorTab::Inspector,
            EditorTab::TimelineSetting,
        ]
        .map(Anchor::tab),
    );

    story
}
