use crate::action::ActionRegistrationExt;
use crate::editing::command::curve_note_track::RemoveCurveNoteTrack;
use crate::editing::command::event::RemoveEvent;
use crate::editing::command::note::RemoveNote;
use crate::editing::command::{CommandSequence, EditorCommand};
use crate::editing::DoCommandEvent;
use crate::hotkey::Hotkey;
use crate::selection::Selected;
use bevy::prelude::*;
use phichain_chart::event::LineEvent;
use phichain_chart::note::Note;
use phichain_game::curve_note_track::CurveNoteTrack;

pub struct DeleteSelectedPlugin;

impl Plugin for DeleteSelectedPlugin {
    fn build(&self, app: &mut App) {
        app.add_action(
            "phichain.delete_selected",
            delete_selected_system,
            Some(Hotkey::new(KeyCode::Backspace, vec![])),
        );
    }
}

fn delete_selected_system(
    mut set: ParamSet<(
        Query<Entity, (With<Selected>, With<Note>)>,
        Query<Entity, (With<Selected>, With<LineEvent>)>,
        Query<Entity, (With<Selected>, With<CurveNoteTrack>)>,
    )>,
    mut events: EventWriter<DoCommandEvent>,
) {
    let mut sequence = CommandSequence(vec![]);
    for note in &set.p0() {
        sequence
            .0
            .push(EditorCommand::RemoveNote(RemoveNote::new(note)));
    }
    for event in &set.p1() {
        sequence
            .0
            .push(EditorCommand::RemoveEvent(RemoveEvent::new(event)));
    }
    for track in &set.p2() {
        sequence.0.push(EditorCommand::RemoveCurveNoteTrack(
            RemoveCurveNoteTrack::new(track),
        ));
    }

    if !sequence.0.is_empty() {
        events.send(DoCommandEvent(EditorCommand::CommandSequence(sequence)));
    }
}
