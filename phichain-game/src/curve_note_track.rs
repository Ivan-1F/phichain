use crate::GameSet;
use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use phichain_chart::curve_note_track::{generate_notes, CurveNoteTrackOptions};
use phichain_chart::id::CurveNoteTrackId;
use phichain_chart::note::Note;

/// The origin note of a curve note track
#[derive(Debug, Clone, Copy, Component)]
#[relationship(relationship_target = CurveNoteTrackFroms)]
#[require(CurveNoteTrackOptions)]
pub struct CurveNoteTrackFrom(pub Entity);

/// The destination note of a curve note track
#[derive(Debug, Clone, Copy, Component)]
#[relationship(relationship_target = CurveNoteTrackTos)]
#[require(CurveNoteTrackId)]
pub struct CurveNoteTrackTo(pub Entity);

#[derive(Debug, Component)]
#[relationship_target(relationship = CurveNoteTrackFrom, linked_spawn)]
pub struct CurveNoteTrackFroms(Vec<Entity>);

#[derive(Debug, Component)]
#[relationship_target(relationship = CurveNoteTrackTo, linked_spawn)]
pub struct CurveNoteTrackTos(Vec<Entity>);

/// A complete curve note track
#[derive(QueryData)]
#[query_data(mutable)]
pub struct CurveNoteTrack {
    pub options: &'static mut CurveNoteTrackOptions,
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
    query: Query<(&CurveNote, Entity)>,
    mut track_query: Query<(
        CurveNoteTrack,
        &ChildOf,
        Option<&mut CurveNoteCache>,
        Entity,
    )>,
) {
    for (track, child_of, cache, entity) in &mut track_query {
        let (Ok(from), Ok(to)) = (note_query.get(track.from.0), note_query.get(track.to.0)) else {
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
                    // TODO bevy-0.16: maybe this is unnecessary now
                    commands
                        .entity(child_of.parent())
                        .detach_children(&[note_entity]);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn spawn_track(world: &mut World, from: Entity, to: Entity) -> Entity {
        world
            .spawn((
                CurveNoteTrackOptions::default(),
                CurveNoteTrackFrom(from),
                CurveNoteTrackTo(to),
            ))
            .id()
    }

    #[test]
    fn two_relationships_coexist_on_one_entity() {
        let mut world = World::new();
        let a = world.spawn_empty().id();
        let b = world.spawn_empty().id();
        let track = spawn_track(&mut world, a, b);

        assert_eq!(world.get::<CurveNoteTrackFrom>(track).unwrap().0, a);
        assert_eq!(world.get::<CurveNoteTrackTo>(track).unwrap().0, b);
        assert!(world.get::<CurveNoteTrackFroms>(a).is_some());
        assert!(world.get::<CurveNoteTrackTos>(b).is_some());
    }

    #[test]
    fn despawning_from_note_cascades_to_track() {
        let mut world = World::new();
        let a = world.spawn_empty().id();
        let b = world.spawn_empty().id();
        let track = spawn_track(&mut world, a, b);

        world.entity_mut(a).despawn();

        assert!(world.get_entity(track).is_err());
    }

    #[test]
    fn despawning_to_note_cascades_to_track() {
        let mut world = World::new();
        let a = world.spawn_empty().id();
        let b = world.spawn_empty().id();
        let track = spawn_track(&mut world, a, b);

        world.entity_mut(b).despawn();

        assert!(world.get_entity(track).is_err());
    }

    #[test]
    fn one_note_can_be_referenced_by_many_tracks() {
        let mut world = World::new();
        let a = world.spawn_empty().id();
        let b = world.spawn_empty().id();
        let c = world.spawn_empty().id();
        let t1 = spawn_track(&mut world, a, b);
        let t2 = spawn_track(&mut world, a, c);

        world.entity_mut(a).despawn();

        assert!(world.get_entity(t1).is_err());
        assert!(world.get_entity(t2).is_err());
    }

    #[test]
    fn degenerate_from_equals_to_despawns_once() {
        let mut world = World::new();
        let a = world.spawn_empty().id();
        let track = spawn_track(&mut world, a, a);

        world.entity_mut(a).despawn();

        assert!(world.get_entity(track).is_err());
    }

    #[test]
    fn despawning_track_unlinks_from_targets() {
        let mut world = World::new();
        let a = world.spawn_empty().id();
        let b = world.spawn_empty().id();
        let track = spawn_track(&mut world, a, b);

        world.entity_mut(track).despawn();

        // empty relationship target collections are removed entirely
        assert!(world.get::<CurveNoteTrackFroms>(a).is_none());
        assert!(world.get_entity(a).is_ok());
        assert!(world.get_entity(b).is_ok());
    }

    #[test]
    fn from_requires_options() {
        let mut world = World::new();
        let a = world.spawn_empty().id();
        let entity = world.spawn(CurveNoteTrackFrom(a)).id();

        assert!(world.get::<CurveNoteTrackOptions>(entity).is_some());
    }

    #[test]
    fn to_requires_id() {
        let mut world = World::new();
        let a = world.spawn_empty().id();
        let entity = world.spawn(CurveNoteTrackTo(a)).id();

        assert!(world.get::<CurveNoteTrackId>(entity).is_some());
    }

    #[test]
    fn explicit_components_win_over_required_defaults() {
        let mut world = World::new();
        let a = world.spawn_empty().id();
        let options = CurveNoteTrackOptions {
            density: 24,
            ..Default::default()
        };
        let id = CurveNoteTrackId::new();
        let entity = world
            .spawn((
                options.clone(),
                id,
                CurveNoteTrackFrom(a),
                CurveNoteTrackTo(a),
            ))
            .id();

        assert_eq!(
            world.get::<CurveNoteTrackOptions>(entity).unwrap().density,
            24
        );
        assert_eq!(world.get::<CurveNoteTrackId>(entity).unwrap(), &id);
    }
}
