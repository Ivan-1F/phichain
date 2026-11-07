use crate::Derived;
use crate::GameSet;
use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use phichain_chart::curve_note_track::{generate_notes, CurveNoteTrackOptions};
use phichain_chart::id::CurveNoteTrackId;
use phichain_chart::note::Note;

/// The origin note of a curve note track
#[derive(Debug, Clone, Copy, PartialEq, Component, Reflect)]
#[reflect(Component, Clone, PartialEq, Debug)]
#[relationship(relationship_target = CurveNoteTracksFrom)]
#[require(CurveNoteTrackOptions)]
pub struct CurveNoteTrackFrom(#[entities] pub Entity);

/// The destination note of a curve note track
#[derive(Debug, Clone, Copy, PartialEq, Component, Reflect)]
#[reflect(Component, Clone, PartialEq, Debug)]
#[relationship(relationship_target = CurveNoteTracksTo)]
#[require(CurveNoteTrackId)]
pub struct CurveNoteTrackTo(#[entities] pub Entity);

#[derive(Debug, Component)]
#[relationship_target(relationship = CurveNoteTrackFrom, linked_spawn)]
pub struct CurveNoteTracksFrom(Vec<Entity>);

#[derive(Debug, Component)]
#[relationship_target(relationship = CurveNoteTrackTo, linked_spawn)]
pub struct CurveNoteTracksTo(Vec<Entity>);

/// A complete curve note track
#[derive(QueryData)]
pub struct CurveNoteTrack {
    pub options: &'static CurveNoteTrackOptions,
    pub from: &'static CurveNoteTrackFrom,
    pub to: &'static CurveNoteTrackTo,
    pub id: &'static CurveNoteTrackId,
}

pub struct CurveNoteTrackPlugin;

impl Plugin for CurveNoteTrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_curve_note_track_system.in_set(GameSet));
    }
}

#[derive(Component)]
pub struct CurveNoteCache(Vec<Note>);

/// A note generated from a curve note track; inner value is the track entity
#[derive(Component)]
#[relationship(relationship_target = CurveNotes)]
#[require(Derived)]
pub struct CurveNote(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = CurveNote, linked_spawn)]
pub struct CurveNotes(Vec<Entity>);

/// For each existing [`CurveNoteTrack`], calculate its note sequence and compare it with the cached version.
///
/// If the cache is outdated, invalidate the cache, despawn all associated [`CurveNote`] instances and generate new ones
pub fn update_curve_note_track_system(
    mut commands: Commands,
    note_query: Query<(&Note, &ChildOf)>,
    mut track_query: Query<
        (
            CurveNoteTrack,
            Option<&CurveNotes>,
            Option<&mut CurveNoteCache>,
            Entity,
        ),
        With<ChildOf>,
    >,
) {
    for (track, curve_notes, cache, entity) in &mut track_query {
        let (Ok(from), Ok(to)) = (note_query.get(track.from.0), note_query.get(track.to.0)) else {
            continue;
        };

        let notes = generate_notes(*from.0, *to.0, track.options);

        let update = match cache {
            None => {
                commands
                    .entity(entity)
                    .insert(CurveNoteCache(notes.clone()));
                true
            }
            Some(mut cache) => {
                if cache.0 != notes {
                    cache.0 = notes.clone();
                    true
                } else {
                    false
                }
            }
        };

        if update {
            if let Some(curve_notes) = curve_notes {
                for note_entity in curve_notes.iter() {
                    commands.entity(note_entity).despawn();
                }
            }
            commands.entity(from.1.parent()).with_children(|p| {
                for note in notes {
                    p.spawn((note, CurveNote(entity)));
                }
            });
        }
    }
}
