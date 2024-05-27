use crate::chart::note::Note;
use crate::selection::Selected;
use crate::tab::timeline::TimelineSettings;
use bevy::prelude::*;
use num::{FromPrimitive, Rational32};

pub struct MoveNotePlugin;

impl Plugin for MoveNotePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_note_system);
    }
}

fn move_note_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    timeline_settings: Res<TimelineSettings>,
    mut selected_notes: Query<&mut Note, With<Selected>>,
) {
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        if let Some(start) = selected_notes.iter().min_by_key(|note| note.beat) {
            let to =
                timeline_settings.attach((start.beat + timeline_settings.minimum_beat()).value());
            let delta = to - start.beat;
            for mut note in selected_notes.iter_mut() {
                note.beat = note.beat + delta;
            }
        }
    } else if keyboard.just_pressed(KeyCode::ArrowDown) {
        if let Some(start) = selected_notes.iter().max_by_key(|note| note.beat) {
            let to =
                timeline_settings.attach((start.beat - timeline_settings.minimum_beat()).value());
            let delta = to - start.beat;
            for mut note in selected_notes.iter_mut() {
                note.beat = note.beat + delta;
            }
        }
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) {
        if let Some(start) = selected_notes
            .iter()
            .min_by_key(|note| Rational32::from_f32(note.x))
        {
            let to = timeline_settings.attach_x(start.x - timeline_settings.minimum_lane());
            let delta = to - start.x;
            for mut note in selected_notes.iter_mut() {
                note.x += delta;
            }
        }
    } else if keyboard.just_pressed(KeyCode::ArrowRight) {
        if let Some(start) = selected_notes
            .iter()
            .max_by_key(|note| Rational32::from_f32(note.x))
        {
            let to = timeline_settings.attach_x(start.x + timeline_settings.minimum_lane());
            let delta = to - start.x;
            for mut note in selected_notes.iter_mut() {
                note.x += delta;
            }
        }
    }
}
