use crate::editing::command::event::EditEvent;
use crate::editing::command::{CommandSequence, EditorCommand};
use crate::editing::DoCommandEvent;
use crate::selection::Selected;
use crate::tab::timeline::TimelineSettings;
use bevy::prelude::*;
use phichain_chart::event::LineEvent;

pub struct MoveEventPlugin;

impl Plugin for MoveEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_event_system);
    }
}

fn move_event_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    timeline_settings: Res<TimelineSettings>,
    selected_events: Query<(&LineEvent, Entity), With<Selected>>,

    mut event_writer: EventWriter<DoCommandEvent>,
) {
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        if let Some((start, _)) = selected_events
            .iter()
            .min_by_key(|(event, _)| event.start_beat)
        {
            let to = timeline_settings
                .attach((start.start_beat + timeline_settings.minimum_beat()).value());
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
    } else if keyboard.just_pressed(KeyCode::ArrowDown) {
        if let Some((start, _)) = selected_events
            .iter()
            .min_by_key(|(event, _)| event.start_beat)
        {
            let to = timeline_settings
                .attach((start.start_beat - timeline_settings.minimum_beat()).value());
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
    }
}
