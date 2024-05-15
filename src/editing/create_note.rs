use bevy::prelude::*;

use crate::{
    chart::{
        beat::Beat,
        note::{Note, NoteBundle, NoteKind},
    },
    constants::CANVAS_WIDTH,
    selection::SelectedLine,
    tab::timeline::{Timeline, TimelineSettings, TimelineViewport},
    timing::BpmList,
};

pub fn create_note_system(
    mut commands: Commands,
    timeline: Timeline,
    keyboard: Res<ButtonInput<KeyCode>>,

    selected_line: Res<SelectedLine>,

    window_query: Query<&Window>,
    bpm_list: Res<BpmList>,

    timeline_viewport: Res<TimelineViewport>,
    timeline_settings: Res<TimelineSettings>,
) {
    let window = window_query.single();
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let note_timeline_viewport = timeline_viewport.note_timeline_viewport();

    if !note_timeline_viewport.contains(cursor_position) {
        return;
    }

    let mut spawn_note = |kind: NoteKind| {
        let time = timeline.y_to_time(cursor_position.y);
        let mut beat = bpm_list.beat_at(time);
        beat.attach_to_beat_line(timeline_settings.density);

        let x = (cursor_position.x - note_timeline_viewport.min.x) / note_timeline_viewport.width()
            - 0.5;

        commands.entity(selected_line.0).with_children(|parent| {
            parent.spawn(NoteBundle::new(Note::new(
                kind,
                true,
                beat,
                x * CANVAS_WIDTH,
            )));
        });
    };

    if keyboard.just_pressed(KeyCode::KeyQ) {
        spawn_note(NoteKind::Tap);
    }

    if keyboard.just_pressed(KeyCode::KeyW) {
        spawn_note(NoteKind::Drag);
    }

    if keyboard.just_pressed(KeyCode::KeyE) {
        spawn_note(NoteKind::Flick);
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        // TODO: make hold placement done with 2 `R` press
        spawn_note(NoteKind::Hold {
            hold_beat: Beat::ONE,
        });
    }
}
