use crate::notification::{ToastsExt, ToastsStorage};
use crate::selection::Selected;
use crate::GameSet;
use bevy::prelude::*;
use phichain_chart::curve_note_track::generate_notes;
use phichain_chart::note::Note;
use phichain_game::curve_note_track::{update_curve_note_track_system, CurveNote, CurveNoteTrack};

pub struct CurveNoteTrackPlugin;

impl Plugin for CurveNoteTrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            despawn_invalid_curve_note_track_system
                .before(update_curve_note_track_system)
                .in_set(GameSet),
        );
    }
}

fn despawn_invalid_curve_note_track_system(
    mut commands: Commands,
    note_query: Query<(&Note, &Parent)>,
    query: Query<(&CurveNote, Entity)>,
    mut track_query: Query<(&CurveNoteTrack, Option<&Selected>, Entity)>,

    mut toasts: ResMut<ToastsStorage>,
) {
    for (track, selected, entity) in &mut track_query {
        let mut despawn_invalid_track = |message: &str| {
            // despawn all existing `CurveNote` objects associated with it, and despawn the `CurveNoteTrack` itself
            for (note, note_entity) in &query {
                if note.0 == entity {
                    commands.entity(note_entity).despawn();
                }
            }
            commands.entity(entity).despawn();

            toasts.info(t!(message));
        };

        let Some((from, to)) = track.get_entities() else {
            // despawn unselected incomplete track
            if selected.is_none() {
                debug!("despawn unselected incomplete CNT");
                despawn_invalid_track("tab.inspector.curve_note_track.removed.incomplete");
            }
            continue;
        };

        let (Ok(from), Ok(to)) = (note_query.get(from), note_query.get(to)) else {
            // despawn invalid track
            debug!("despawn invalid CNT");
            despawn_invalid_track("tab.inspector.curve_note_track.removed.invalid");
            continue;
        };

        let notes = generate_notes(*from.0, *to.0, &track.options);

        if notes.is_empty() && selected.is_none() {
            // despawn unselected empty tracks
            debug!("despawn unselected empty CNT");
            despawn_invalid_track("tab.inspector.curve_note_track.removed.empty");
            continue;
        }
    }
}
