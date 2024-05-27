use crate::chart::event::LineEvent;
use crate::selection::Selected;
use crate::tab::timeline::TimelineSettings;
use bevy::prelude::*;

pub struct MoveEventPlugin;

impl Plugin for MoveEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_event_system);
    }
}

fn move_event_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    timeline_settings: Res<TimelineSettings>,
    mut selected_events: Query<&mut LineEvent, With<Selected>>,
) {
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        if let Some(start) = selected_events.iter().min_by_key(|note| note.start_beat) {
            let to = timeline_settings
                .attach((start.start_beat + timeline_settings.minimum_beat()).value());
            let delta = to - start.start_beat;
            for mut event in selected_events.iter_mut() {
                event.start_beat += delta;
                event.end_beat += delta;
            }
        }
    } else if keyboard.just_pressed(KeyCode::ArrowDown) {
        if let Some(start) = selected_events.iter().max_by_key(|note| note.start_beat) {
            let to = timeline_settings
                .attach((start.start_beat - timeline_settings.minimum_beat()).value());
            let delta = to - start.start_beat;
            for mut event in selected_events.iter_mut() {
                event.start_beat += delta;
                event.end_beat += delta;
            }
        }
    }
}
