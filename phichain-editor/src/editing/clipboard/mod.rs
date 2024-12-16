use crate::action::ActionRegistrationExt;
use crate::editing::command::event::{CreateEvent, RemoveEvent};
use crate::editing::command::note::{CreateNote, RemoveNote};
use crate::editing::command::{CommandSequence, EditorCommand};
use crate::editing::DoCommandEvent;
use crate::hotkey::modifier::Modifier;
use crate::hotkey::Hotkey;
use crate::selection::{Selected, SelectedLine};
use crate::timeline::TimelineContext;
use crate::utils::convert::BevyEguiConvert;
use bevy::prelude::*;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::event::LineEvent;
use phichain_chart::note::Note;

#[derive(Resource, Default)]
struct EditorClipboard {
    notes: Vec<Note>,
    events: Vec<LineEvent>,
}

impl EditorClipboard {
    fn clear(&mut self) {
        self.notes.clear();
        self.events.clear();
    }
}

pub struct ClipboardPlugin;

impl Plugin for ClipboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorClipboard>()
            .add_action(
                "phichain.copy",
                copy_system,
                Some(Hotkey::new(KeyCode::KeyC, vec![Modifier::Control])),
            )
            .add_action(
                "phichain.cut",
                cut_system,
                Some(Hotkey::new(KeyCode::KeyX, vec![Modifier::Control])),
            )
            .add_action(
                "phichain.paste",
                paste_system,
                Some(Hotkey::new(KeyCode::KeyV, vec![Modifier::Control])),
            );
    }
}

fn copy_system(
    mut clipboard: ResMut<EditorClipboard>,

    note_query: Query<&Note>,
    event_query: Query<&LineEvent>,

    selected_query: Query<Entity, With<Selected>>,
) {
    clipboard.clear();

    for entity in &selected_query {
        if let Ok(note) = note_query.get(entity) {
            clipboard.notes.push(*note);
        } else if let Ok(event) = event_query.get(entity) {
            clipboard.events.push(*event);
        }
    }
}

fn cut_system(
    mut clipboard: ResMut<EditorClipboard>,

    note_query: Query<&Note>,
    event_query: Query<&LineEvent>,

    selected_query: Query<Entity, With<Selected>>,

    mut event_writer: EventWriter<DoCommandEvent>,
) {
    clipboard.clear();

    let mut commands = vec![];

    for entity in &selected_query {
        if let Ok(note) = note_query.get(entity) {
            clipboard.notes.push(*note);
            commands.push(EditorCommand::RemoveNote(RemoveNote::new(entity)));
        } else if let Ok(event) = event_query.get(entity) {
            clipboard.events.push(*event);
            commands.push(EditorCommand::RemoveEvent(RemoveEvent::new(entity)));
        }
    }

    event_writer.send(DoCommandEvent(EditorCommand::CommandSequence(
        CommandSequence(commands),
    )));
}

fn paste_system(
    clipboard: Res<EditorClipboard>,

    window_query: Query<&Window>,

    selected_line: Res<SelectedLine>,

    ctx: TimelineContext,
    bpm_list: Res<BpmList>,

    mut event_writer: EventWriter<DoCommandEvent>,
) {
    let window = window_query.single();
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    if !ctx.viewport.0.contains(cursor_position) {
        return;
    }

    let timeline = ctx
        .settings
        .container
        .allocate(ctx.viewport.0.into_egui())
        .iter()
        .find(|x| x.viewport.x_range().contains(cursor_position.x))
        .map(|x| x.timeline);

    let Some(timeline) = timeline else {
        return;
    };

    let target_line = timeline.line_entity().unwrap_or(selected_line.0);

    let notes = clipboard.notes.to_vec();
    let events = clipboard.events.to_vec();

    if let Some(min_beat) = notes
        .iter()
        .map(|note| note.beat)
        .chain(events.iter().map(|event| event.start_beat))
        .min()
    {
        let time = ctx.y_to_time(cursor_position.y);
        let beat = ctx.settings.attach(bpm_list.beat_at(time).value());

        let delta = beat - min_beat;

        let mut sequence = CommandSequence(vec![]);

        for note in notes {
            let mut new_note = note;
            new_note.beat = note.beat + delta;
            sequence.0.push(EditorCommand::CreateNote(CreateNote::new(
                target_line,
                new_note,
            )));
        }
        for event in events {
            let mut new_event = event;
            new_event.start_beat = event.start_beat + delta;
            new_event.end_beat = event.end_beat + delta;
            sequence.0.push(EditorCommand::CreateEvent(CreateEvent::new(
                target_line,
                new_event,
            )));
        }

        if !sequence.0.is_empty() {
            event_writer.send(DoCommandEvent(EditorCommand::CommandSequence(sequence)));
        }
    }
}
