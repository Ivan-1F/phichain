use crate::capture::locate::by_role;
use crate::capture::story::Story;
use crate::identifier::IntoIdentifier;
use crate::tab::EditorTab;
use bevy_egui::egui::accesskit::Role;
use phichain_chart::project::ProjectMeta;

/// Lists what assistive technology cannot name, with every tab of the editor shown
pub fn audit() -> Story {
    let mut story = Story::new(ProjectMeta::default());
    let mut toggle = |tab: EditorTab| {
        story.click(by_role(Role::Button).name("menu_bar.tabs.title"));
        story.click(by_role(Role::Button).name(format!("tab.{}.title", tab.into_identifier())));
    };
    // closed in the default layout
    toggle(EditorTab::ChartBasicSetting);
    toggle(EditorTab::BpmList);
    toggle(EditorTab::History);
    // behind the timeline in the default layout, so it comes back in front in a window
    toggle(EditorTab::Settings);
    toggle(EditorTab::Settings);

    story.audit("editor");
    story
}
