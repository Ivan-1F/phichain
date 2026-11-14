use super::{
    create_new_project, fill_new_project, new_project_dialog, new_project_field, toggle_tab,
};
use crate::capture::locate::{around, by_role, event_timeline_at, note_timeline_at, panel};
use crate::capture::story::{Anchor, Story};
use crate::tab::EditorTab;
use bevy::input::keyboard::KeyCode;
use bevy_egui::egui::accesskit::Role;
use phichain_chart::event::LineEventKind;

pub fn quick_start() -> Story {
    let mut story = Story::default();

    fill_new_project(&mut story);
    let dialog_button = |name: &str| new_project_dialog().find(by_role(Role::Button).name(name));
    story
        .image("create-project")
        .of(new_project_dialog())
        .anchor(Anchor::new(
            "create_project.music",
            dialog_button("home.create_project.select_music"),
        ))
        .anchor(Anchor::new(
            "create_project.illustration",
            dialog_button("home.create_project.select_illustration"),
        ))
        .anchor(Anchor::new(
            "create_project.meta",
            around(super::PROJECT_INFO.map(|(field, _)| new_project_field(field))),
        ))
        .anchor(Anchor::new(
            "create_project.create",
            dialog_button("home.create_project.create"),
        ));
    create_new_project(&mut story);

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

    toggle_tab(&mut story, EditorTab::BpmList);
    story
        .image("bpm-list")
        .anchor(Anchor::new("bpm_list.panel", panel(EditorTab::BpmList)))
        .anchor(Anchor::new(
            "bpm_list.point_0.bpm",
            panel(EditorTab::BpmList)
                .find(by_role(Role::SpinButton).name("tab.bpm_list.point.bpm")),
        ));
    toggle_tab(&mut story, EditorTab::BpmList);

    // the demos use beats 1.5 to 3.5, which fit at a smaller zoom from beat 1
    story.click(
        panel(EditorTab::TimelineSetting)
            .find(by_role(Role::SpinButton).name("tab.timeline_setting.zoom")),
    );
    story.type_text("1");
    story.press(KeyCode::Enter);
    story.seek(1.0);

    story.video("place-notes", |s| {
        s.wait(0.75);
        // a little off the grid lines, so every note visibly snaps
        for (beat, lane, key) in [
            (1.97, 2, KeyCode::KeyQ),
            (2.28, 4, KeyCode::KeyW),
            (2.47, 6, KeyCode::KeyE),
        ] {
            s.move_to(note_timeline_at(beat, lane));
            s.press(key);
            s.wait(0.75);
        }
        s.wait(0.75);
    });

    story.video("place-hold", |s| {
        s.step(1, |s| {
            s.wait(0.6);
            // starts with the tap, so both show they are hit together
            s.move_to(note_timeline_at(1.97, 8));
            s.press(KeyCode::KeyR);
            s.wait(0.5);
        });
        s.step(2, |s| {
            s.move_to(note_timeline_at(3.54, 8));
        });
        s.step(3, |s| {
            s.press(KeyCode::KeyR);
            s.wait(1.5);
        });
    });

    let end_value = panel(EditorTab::Inspector)
        .find(by_role(Role::SpinButton).name("tab.inspector.single_event.end_value"));
    story.video("create-event", |s| {
        s.step(1, |s| {
            s.wait(0.6);
            s.move_to(event_timeline_at(LineEventKind::Y, 1.55));
            s.press(KeyCode::KeyR);
            s.wait(0.5);
        });
        s.step(2, |s| {
            s.move_to(event_timeline_at(LineEventKind::Y, 3.45));
            s.press(KeyCode::KeyR);
            s.wait(0.75);
        });
        s.step(3, |s| {
            s.click(event_timeline_at(LineEventKind::Y, 2.5));
            s.wait(0.5);
            s.highlight(end_value.clone(), 3.0);
            s.click(end_value);
            s.wait(0.4);
            s.type_text("-300");
            s.wait(0.3);
            s.press(KeyCode::Enter);
            s.wait(1.6);
        });
    });

    story.seek(0.0);
    story.video("preview-result", |s| {
        s.wait(0.7);
        s.press(KeyCode::Space);
        s.wait(2.5);
        s.press(KeyCode::Space);
        s.wait(1.3);
    });

    story
}
