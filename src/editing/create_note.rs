use bevy::prelude::*;

use crate::editing::command::note::CreateNote;
use crate::editing::command::EditorCommand;
use crate::editing::DoCommandEvent;
use crate::{
    chart::{
        beat::Beat,
        note::{Note, NoteKind},
    },
    constants::CANVAS_WIDTH,
    selection::SelectedLine,
    tab::timeline::{Timeline, TimelineSettings, TimelineViewport},
    timing::BpmList,
};

#[allow(clippy::too_many_arguments)]
pub fn create_note_system(
    timeline: Timeline,
    keyboard: Res<ButtonInput<KeyCode>>,

    selected_line: Res<SelectedLine>,

    window_query: Query<&Window>,
    bpm_list: Res<BpmList>,

    timeline_viewport: Res<TimelineViewport>,
    timeline_settings: Res<TimelineSettings>,

    mut event: EventWriter<DoCommandEvent>,
) {
    let window = window_query.single();
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let note_timeline_viewport = timeline_viewport.note_timeline_viewport();

    if !note_timeline_viewport.contains(cursor_position) {
        return;
    }

    let mut create_note = |kind: NoteKind| {
        let time = timeline.y_to_time(cursor_position.y);
        let mut beat = bpm_list.beat_at(time);
        beat.attach_to_beat_line(timeline_settings.density);

        let x = (cursor_position.x - note_timeline_viewport.min.x) / note_timeline_viewport.width();

        let lane_percents = timeline_settings.lane_percents();

        let x = lane_percents
            .iter()
            .map(|p| (p, (p - x).abs()))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
            .0;

        let x = x - 0.5;

        let note = Note::new(kind, true, beat, x * CANVAS_WIDTH, 1.0);

        event.send(DoCommandEvent(EditorCommand::CreateNote(CreateNote::new(
            selected_line.0,
            note,
        ))));
    };

    if keyboard.just_pressed(KeyCode::KeyQ) {
        create_note(NoteKind::Tap);
    }

    if keyboard.just_pressed(KeyCode::KeyW) {
        create_note(NoteKind::Drag);
    }

    if keyboard.just_pressed(KeyCode::KeyE) {
        create_note(NoteKind::Flick);
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        // TODO: make hold placement done with 2 `R` press
        create_note(NoteKind::Hold {
            hold_beat: Beat::ONE,
        });
    }
}
