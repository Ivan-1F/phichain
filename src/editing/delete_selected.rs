use crate::action::ActionRegistrationExt;
use crate::chart::event::LineEvent;
use crate::chart::note::Note;
use crate::editing::command::event::RemoveEvent;
use crate::editing::command::note::RemoveNote;
use crate::editing::command::{CommandSequence, EditorCommand};
use crate::editing::DoCommandEvent;
use crate::hotkey::HotkeyRegistrationExt;
use crate::selection::Selected;
use bevy::prelude::*;

pub struct DeleteSelectedPlugin;

impl Plugin for DeleteSelectedPlugin {
    fn build(&self, app: &mut App) {
        app.register_action("phichain.delete", delete_selected_system)
            .register_hotkey("phichain.delete", vec![KeyCode::Backspace]);
    }
}

fn delete_selected_system(
    note_query: Query<Entity, (With<Selected>, With<Note>, Without<LineEvent>)>,
    event_query: Query<Entity, (With<Selected>, With<LineEvent>, Without<Note>)>,
    mut events: EventWriter<DoCommandEvent>,
) {
    let mut sequence = CommandSequence(vec![]);
    for note in &note_query {
        sequence
            .0
            .push(EditorCommand::RemoveNote(RemoveNote::new(note)));
    }
    for event in &event_query {
        sequence
            .0
            .push(EditorCommand::RemoveEvent(RemoveEvent::new(event)));
    }

    events.send(DoCommandEvent(EditorCommand::CommandSequence(sequence)));
}
