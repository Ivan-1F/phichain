use crate::editing::command::note::EditNote;
use crate::editing::command::{CommandSequence, EditorCommand};
use crate::editing::DoCommandEvent;
use crate::selection::Selected;
use crate::tab::timeline::TimelineSettings;
use bevy::prelude::*;
use num::{FromPrimitive, Rational32};
use phichain_chart::note::Note;

pub struct MoveNotePlugin;

impl Plugin for MoveNotePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_note_system);
    }
}

fn move_note_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    timeline_settings: Res<TimelineSettings>,
    selected_notes: Query<(&Note, Entity), With<Selected>>,

    mut event_writer: EventWriter<DoCommandEvent>,
) {
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        if let Some((start, _)) = selected_notes.iter().min_by_key(|(note, _)| note.beat) {
            let to =
                timeline_settings.attach((start.beat + timeline_settings.minimum_beat()).value());
            let delta = to - start.beat;
            event_writer.send(DoCommandEvent(EditorCommand::CommandSequence(
                CommandSequence(
                    selected_notes
                        .iter()
                        .map(|(note, entity)| {
                            let new_note = Note {
                                beat: note.beat + delta,
                                ..*note
                            };
                            EditorCommand::EditNote(EditNote::new(entity, *note, new_note))
                        })
                        .collect(),
                ),
            )));
        }
    } else if keyboard.just_pressed(KeyCode::ArrowDown) {
        if let Some((start, _)) = selected_notes.iter().min_by_key(|(note, _)| note.beat) {
            let to =
                timeline_settings.attach((start.beat - timeline_settings.minimum_beat()).value());
            let delta = to - start.beat;
            event_writer.send(DoCommandEvent(EditorCommand::CommandSequence(
                CommandSequence(
                    selected_notes
                        .iter()
                        .map(|(note, entity)| {
                            let new_note = Note {
                                beat: note.beat + delta,
                                ..*note
                            };
                            EditorCommand::EditNote(EditNote::new(entity, *note, new_note))
                        })
                        .collect(),
                ),
            )));
        }
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) {
        if let Some((start, _)) = selected_notes
            .iter()
            .min_by_key(|(note, _)| Rational32::from_f32(note.x))
        {
            let to = timeline_settings.attach_x(start.x - timeline_settings.minimum_lane());
            let delta = to - start.x;
            event_writer.send(DoCommandEvent(EditorCommand::CommandSequence(
                CommandSequence(
                    selected_notes
                        .iter()
                        .map(|(note, entity)| {
                            let new_note = Note {
                                x: note.x + delta,
                                ..*note
                            };
                            EditorCommand::EditNote(EditNote::new(entity, *note, new_note))
                        })
                        .collect(),
                ),
            )));
        }
    } else if keyboard.just_pressed(KeyCode::ArrowRight) {
        if let Some((start, _)) = selected_notes
            .iter()
            .max_by_key(|(note, _)| Rational32::from_f32(note.x))
        {
            let to = timeline_settings.attach_x(start.x + timeline_settings.minimum_lane());
            let delta = to - start.x;
            event_writer.send(DoCommandEvent(EditorCommand::CommandSequence(
                CommandSequence(
                    selected_notes
                        .iter()
                        .map(|(note, entity)| {
                            let new_note = Note {
                                x: note.x + delta,
                                ..*note
                            };
                            EditorCommand::EditNote(EditNote::new(entity, *note, new_note))
                        })
                        .collect(),
                ),
            )));
        }
    }
}
