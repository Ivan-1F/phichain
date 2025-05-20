use crate::action::ActionRegistrationExt;
use crate::editing::command::event::EditEvent;
use crate::editing::command::note::EditNote;
use crate::editing::command::{CommandSequence, EditorCommand};
use crate::editing::DoCommandEvent;
use crate::hotkey::Hotkey;
use crate::selection::Selected;
use crate::timeline::settings::TimelineSettings;
use bevy::prelude::*;
use num::{FromPrimitive, Rational32};
use phichain_chart::event::LineEvent;
use phichain_chart::note::Note;

pub struct MovingPlugin;

impl Plugin for MovingPlugin {
    fn build(&self, app: &mut App) {
        app.add_action(
            "phichain.move_up",
            move_up_system,
            Some(Hotkey::new(KeyCode::ArrowUp, vec![])),
        )
        .add_action(
            "phichain.move_down",
            move_down_system,
            Some(Hotkey::new(KeyCode::ArrowDown, vec![])),
        )
        .add_action(
            "phichain.move_left",
            move_left_system,
            Some(Hotkey::new(KeyCode::ArrowLeft, vec![])),
        )
        .add_action(
            "phichain.move_right",
            move_right_system,
            Some(Hotkey::new(KeyCode::ArrowRight, vec![])),
        );
    }
}

fn move_up_system(
    timeline_settings: Res<TimelineSettings>,
    selected_notes: Query<(&Note, Entity), With<Selected>>,
    selected_events: Query<(&LineEvent, Entity), With<Selected>>,
    mut event_writer: EventWriter<DoCommandEvent>,
) -> Result {
    if let Some((start, _)) = selected_notes.iter().min_by_key(|(note, _)| note.beat) {
        let to = timeline_settings.attach((start.beat + timeline_settings.minimum_beat()).value());
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

    if let Some((start, _)) = selected_events
        .iter()
        .min_by_key(|(event, _)| event.start_beat)
    {
        let to =
            timeline_settings.attach((start.start_beat + timeline_settings.minimum_beat()).value());
        let delta = to - start.start_beat;
        event_writer.send(DoCommandEvent(EditorCommand::CommandSequence(
            CommandSequence(
                selected_events
                    .iter()
                    .map(|(event, entity)| {
                        let new_event = LineEvent {
                            start_beat: event.start_beat + delta,
                            end_beat: event.end_beat + delta,
                            ..*event
                        };
                        EditorCommand::EditEvent(EditEvent::new(entity, *event, new_event))
                    })
                    .collect(),
            ),
        )));
    }

    Ok(())
}

fn move_down_system(
    timeline_settings: Res<TimelineSettings>,
    selected_notes: Query<(&Note, Entity), With<Selected>>,
    selected_events: Query<(&LineEvent, Entity), With<Selected>>,
    mut event_writer: EventWriter<DoCommandEvent>,
) -> Result {
    if let Some((start, _)) = selected_notes.iter().min_by_key(|(note, _)| note.beat) {
        let to = timeline_settings.attach((start.beat - timeline_settings.minimum_beat()).value());
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

    if let Some((start, _)) = selected_events
        .iter()
        .min_by_key(|(event, _)| event.start_beat)
    {
        let to =
            timeline_settings.attach((start.start_beat - timeline_settings.minimum_beat()).value());
        let delta = to - start.start_beat;
        event_writer.send(DoCommandEvent(EditorCommand::CommandSequence(
            CommandSequence(
                selected_events
                    .iter()
                    .map(|(event, entity)| {
                        let new_event = LineEvent {
                            start_beat: event.start_beat + delta,
                            end_beat: event.end_beat + delta,
                            ..*event
                        };
                        EditorCommand::EditEvent(EditEvent::new(entity, *event, new_event))
                    })
                    .collect(),
            ),
        )));
    }

    Ok(())
}

fn move_left_system(
    timeline_settings: Res<TimelineSettings>,
    selected_notes: Query<(&Note, Entity), With<Selected>>,
    mut event_writer: EventWriter<DoCommandEvent>,
) -> Result {
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

    Ok(())
}

fn move_right_system(
    timeline_settings: Res<TimelineSettings>,
    selected_notes: Query<(&Note, Entity), With<Selected>>,
    mut event_writer: EventWriter<DoCommandEvent>,
) -> Result {
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

    Ok(())
}
