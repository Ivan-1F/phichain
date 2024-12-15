use crate::action::ActionRegistrationExt;
use crate::editing::command::event::RemoveEvent;
use crate::editing::command::note::RemoveNote;
use crate::editing::command::{CommandSequence, EditorCommand};
use crate::editing::DoCommandEvent;
use crate::hotkey::Hotkey;
use crate::selection::Selected;
use bevy::prelude::*;
use phichain_chart::event::LineEvent;
use phichain_chart::note::Note;

pub struct DeleteSelectedPlugin;

impl Plugin for DeleteSelectedPlugin {
    fn build(&self, app: &mut App) {
        app.register_action(
            "phichain.delete_selected",
            delete_selected_system,
            Some(Hotkey::new(KeyCode::Backspace, vec![])),
        );
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

    if !sequence.0.is_empty() {
        events.send(DoCommandEvent(EditorCommand::CommandSequence(sequence)));
    }
}
