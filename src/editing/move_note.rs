use crate::beat;
use crate::chart::beat::Beat;
use crate::chart::note::Note;
use crate::selection::Selected;
use crate::tab::timeline::TimelineSettings;
use bevy::prelude::*;

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
            let delta = timeline_settings
                .attach((start.beat + beat!(1, timeline_settings.density)).value())
                - start.beat;
            for mut note in selected_notes.iter_mut() {
                note.beat = note.beat + delta;
            }
        }
    } else if keyboard.just_pressed(KeyCode::ArrowDown) {
        if let Some(start) = selected_notes.iter().max_by_key(|note| note.beat) {
            let delta = timeline_settings
                .attach((start.beat - beat!(1, timeline_settings.density)).value())
                - start.beat;
            for mut note in selected_notes.iter_mut() {
                note.beat = note.beat + delta;
            }
        }
    }
}
