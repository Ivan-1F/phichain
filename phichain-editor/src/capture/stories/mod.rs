mod audit;
mod quick_start;

use super::locate::{by_role, Query};
use super::story::Story;
use super::{ILLUSTRATION, MUSIC, PROJECT};
use crate::identifier::IntoIdentifier;
use crate::tab::EditorTab;
use bevy_egui::egui::accesskit::Role;

/// Named after the documentation page they capture for, which is where `scripts/capture.sh` puts their media
///
/// Stories without a page, such as the audit, are only run by hand.
const STORIES: &[(&str, fn() -> Story)] = &[
    ("quick-start", quick_start::quick_start),
    ("audit", audit::audit),
];

pub fn find(name: &str) -> Option<Story> {
    STORIES
        .iter()
        .find(|(story, _)| *story == name)
        .map(|(_, build)| build())
}

pub fn names() -> Vec<&'static str> {
    STORIES.iter().map(|(name, _)| *name).collect()
}

/// Opens a closed tab from the window menu, or closes an open one
fn toggle_tab(story: &mut Story, tab: EditorTab) {
    story.click(by_role(Role::Button).name("menu_bar.tabs.title"));
    story.click(by_role(Role::Button).name(format!("tab.{}.title", tab.into_identifier())));
}

/// The create project dialog of the home screen
fn new_project_dialog() -> Query {
    by_role(Role::Window).name("home.create_project.label")
}

/// The demo project as typed into the create project dialog
const PROJECT_INFO: [(&str, &str); 5] = [
    ("name", "Quick Start"),
    ("level", "IN Lv.12"),
    ("composer", "Composer"),
    ("charter", "Charter"),
    ("illustrator", "Illustrator"),
];

/// Open the create project dialog from the home screen and fill it in for the demo project
fn fill_new_project(story: &mut Story) {
    story.click(by_role(Role::Button).name("home.create_project.create"));
    let dialog_button = |name: &str| new_project_dialog().find(by_role(Role::Button).name(name));
    story.click(dialog_button("home.create_project.select_music"));
    story.choose(MUSIC);
    story.click(dialog_button("home.create_project.select_illustration"));
    story.choose(ILLUSTRATION);
    for (field, text) in PROJECT_INFO {
        story.click(new_project_field(field));
        story.type_text(text);
    }
}

fn new_project_field(field: &str) -> Query {
    new_project_dialog().find(by_role(Role::TextInput).name(format!("home.create_project.{field}")))
}

/// Create the project filled in, in the empty project folder, and wait for it to open
fn create_new_project(story: &mut Story) {
    story
        .click(new_project_dialog().find(by_role(Role::Button).name("home.create_project.create")));
    story.choose(PROJECT);
}
