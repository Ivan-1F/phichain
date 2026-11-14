use super::toggle_tab;
use crate::capture::story::Story;
use crate::tab::EditorTab;

/// Lists what assistive technology cannot name, with every tab of the editor shown
pub fn audit() -> Story {
    let mut story = Story::default();
    story.audit("home");
    super::fill_new_project(&mut story);
    story.audit("create-project");
    super::create_new_project(&mut story);

    // closed in the default layout
    toggle_tab(&mut story, EditorTab::ChartBasicSetting);
    toggle_tab(&mut story, EditorTab::BpmList);
    toggle_tab(&mut story, EditorTab::History);
    // behind the timeline in the default layout, so it comes back in front in a window
    toggle_tab(&mut story, EditorTab::Settings);
    toggle_tab(&mut story, EditorTab::Settings);

    story.audit("editor");
    story
}
