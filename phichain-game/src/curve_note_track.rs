use crate::GameSet;
use bevy::prelude::*;
use phichain_chart::curve_note_track::{generate_notes, CurveNoteTrackOptions};
use phichain_chart::note::{Note, NoteBundle};

/// Represents a curve note track
#[derive(Debug, Clone, Component)]
pub struct CurveNoteTrack {
    pub from: Option<Entity>,
    pub to: Option<Entity>,

    pub options: CurveNoteTrackOptions,
}

impl CurveNoteTrack {
    // TODO: rename to `start` to avoid confusion with the `From<T>` trait
    pub fn from(entity: Entity) -> Self {
        Self {
            from: Some(entity),
            to: None,

            options: Default::default(),
        }
    }

    pub fn to(&mut self, entity: Entity) {
        self.to = Some(entity);
    }

    /// Return the [`Entity`] of the origin and the destination
    ///
    /// If one of them is missing, return a [`None`], otherwise a [`Some`]
    pub fn get_entities(&self) -> Option<(Entity, Entity)> {
        if let (Some(from), Some(to)) = (self.from, self.to) {
            Some((from, to))
        } else {
            None
        }
    }
}

pub struct CurveNoteTrackPlugin;

impl Plugin for CurveNoteTrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_curve_note_track_system,
                despawn_dangle_curve_note_system,
            )
                .in_set(GameSet),
        );
    }
}

#[derive(Component)]
pub struct CurveNoteCache(Vec<Note>);

/// Inner value is the attached entity ID of [`CurveNoteTrack`]
#[derive(Component)]
pub struct CurveNote(pub Entity);

/// For each existing [`CurveNoteTrack`], calculate its note sequence and compare it with the cached version.
///
/// If the cache is outdated, invalidate the cache, despawn all associated [`CurveNote`] instances and generate new ones
pub fn update_curve_note_track_system(
    mut commands: Commands,
    note_query: Query<(&Note, &Parent)>,
    query: Query<(&CurveNote, Entity)>,
    mut track_query: Query<(
        &CurveNoteTrack,
        &Parent,
        Option<&mut CurveNoteCache>,
        Entity,
    )>,
) {
    for (track, parent, cache, entity) in &mut track_query {
        let Some((from, to)) = track.get_entities() else {
            continue;
        };

        let (Ok(from), Ok(to)) = (note_query.get(from), note_query.get(to)) else {
            continue;
        };

        let notes = generate_notes(*from.0, *to.0, &track.options);

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
            for (note, note_entity) in &query {
                if note.0 == entity {
                    // despawning children does not remove references for parent
                    // https://github.com/bevyengine/bevy/issues/12235
                    commands
                        .entity(parent.get())
                        .remove_children(&[note_entity]);
                    commands.entity(note_entity).despawn();
                }
            }
            commands.entity(from.1.get()).with_children(|p| {
                for note in notes {
                    p.spawn((NoteBundle::new(note), CurveNote(entity)));
                }
            });
        }
    }
}

/// Search for [`CurveNote`] with an invalid associated [`CurveNoteTrack`] and despawn them
pub fn despawn_dangle_curve_note_system(
    mut commands: Commands,
    query: Query<(Entity, &CurveNote)>,
    track_query: Query<&CurveNoteTrack>,
) {
    for (entity, note) in &query {
        if track_query.get(note.0).is_err() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
