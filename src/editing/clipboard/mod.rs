use crate::action::ActionRegistrationExt;
use crate::chart::event::LineEvent;
use crate::chart::note::Note;
use crate::editing::command::event::CreateEvent;
use crate::editing::command::note::CreateNote;
use crate::editing::command::{CommandSequence, EditorCommand};
use crate::editing::DoCommandEvent;
use crate::hotkey::HotkeyRegistrationExt;
use crate::selection::{Selected, SelectedLine};
use crate::tab::timeline::{Timeline, TimelineSettings, TimelineViewport};
use crate::timing::BpmList;
use crate::utils::compat::ControlKeyExt;
use bevy::prelude::*;

#[derive(Resource, Default)]
struct EditorClipboard(pub Vec<Entity>);

pub struct ClipboardPlugin;

impl Plugin for ClipboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorClipboard>()
            .register_action("phichain.copy", copy_system)
            .register_hotkey("phichain.copy", vec![KeyCode::control(), KeyCode::KeyC])
            .register_action("phichain.paste", paste_system)
            .register_hotkey("phichain.paste", vec![KeyCode::control(), KeyCode::KeyV]);
    }
}

fn copy_system(
    mut clipboard: ResMut<EditorClipboard>,
    selected_query: Query<Entity, With<Selected>>,
) {
    clipboard.0 = selected_query.iter().collect();
}

fn paste_system(
    clipboard: Res<EditorClipboard>,

    note_query: Query<&Note>,
    event_query: Query<&LineEvent>,

    window_query: Query<&Window>,

    selected_line: Res<SelectedLine>,

    timeline: Timeline,
    timeline_viewport: Res<TimelineViewport>,
    bpm_list: Res<BpmList>,
    timeline_settings: Res<TimelineSettings>,

    mut event_writer: EventWriter<DoCommandEvent>,
) {
    let window = window_query.single();
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    if !timeline_viewport.0.contains(cursor_position) {
        return;
    }

    let mut notes = Vec::new();
    let mut events = Vec::new();

    for entity in &clipboard.0 {
        if let Ok(note) = note_query.get(*entity) {
            notes.push(note);
        } else if let Ok(event) = event_query.get(*entity) {
            events.push(event);
        }
    }

    if let Some(min_beat) = notes
        .iter()
        .map(|note| note.beat)
        .chain(events.iter().map(|event| event.start_beat))
        .min()
    {
        let time = timeline.y_to_time(cursor_position.y);
        let beat = timeline_settings.attach(bpm_list.beat_at(time).value());

        let delta = beat - min_beat;

        let mut sequence = CommandSequence(vec![]);

        for note in notes {
            let mut new_note = *note;
            new_note.beat = note.beat + delta;
            sequence.0.push(EditorCommand::CreateNote(CreateNote::new(
                selected_line.0,
                new_note,
            )));
        }
        for event in events {
            let mut new_event = *event;
            new_event.start_beat = event.start_beat + delta;
            new_event.end_beat = event.end_beat + delta;
            sequence.0.push(EditorCommand::CreateEvent(CreateEvent::new(
                selected_line.0,
                new_event,
            )));
        }

        if !sequence.0.is_empty() {
            event_writer.send(DoCommandEvent(EditorCommand::CommandSequence(sequence)));
        }
    }
}
