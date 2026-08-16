use crate::action::ActionRegistrationExt;
use crate::editing::command::curve_note_track::RemoveCurveNoteTrack;
use crate::editing::command::event::RemoveEvent;
use crate::editing::command::note::RemoveNote;
use crate::editing::command::{CommandSequence, EditorCommand};
use crate::editing::DoCommand;
use crate::hotkey::Hotkey;
use crate::selection::Selected;
use bevy::prelude::*;
use phichain_chart::event::LineEvent;
use phichain_chart::note::Note;
use phichain_game::curve_note_track::{CurveNoteTrackTo, CurveNoteTracksFrom, CurveNoteTracksTo};

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
        Query<Entity, (With<Selected>, With<CurveNoteTrackTo>)>,
        Query<(Option<&CurveNoteTracksFrom>, Option<&CurveNoteTracksTo>)>,
    )>,
    mut events: MessageWriter<DoCommand>,
) -> Result {
    let mut sequence = CommandSequence(vec![]);

    let notes: Vec<Entity> = set.p0().iter().collect();
    let events_: Vec<Entity> = set.p1().iter().collect();

    // tracks referencing a deleted note go first: their links must be
    // snapshotted before the note's tombstone strips them
    let mut tracks: Vec<Entity> = set.p2().iter().collect();
    for note in &notes {
        if let Ok((froms, tos)) = set.p3().get(*note) {
            if let Some(froms) = froms {
                tracks.extend(froms.iter());
            }
            if let Some(tos) = tos {
                tracks.extend(tos.iter());
            }
        }
    }
    tracks.sort();
    tracks.dedup();
    for track in tracks {
        sequence.0.push(EditorCommand::RemoveCurveNoteTrack(
            RemoveCurveNoteTrack::new(track),
        ));
    }

    for note in notes {
        sequence
            .0
            .push(EditorCommand::RemoveNote(RemoveNote::new(note)));
    }
    for event in events_ {
        sequence
            .0
            .push(EditorCommand::RemoveEvent(RemoveEvent::new(event)));
    }

    if !sequence.0.is_empty() {
        events.write(DoCommand(EditorCommand::CommandSequence(sequence)));
    }

    Ok(())
}
