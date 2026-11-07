use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use phichain_chart::beat;
use phichain_chart::beat::Beat;
use phichain_chart::event::{EventEvaluationResult, LineEvent, LineEventKind};
use std::collections::BTreeMap;

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, Clone, PartialEq, Debug)]
#[relationship(relationship_target = Events)]
pub struct EventOf(#[entities] pub Entity);

impl EventOf {
    /// The target entity of this event entity.
    #[inline]
    pub fn target(&self) -> Entity {
        self.0
    }
}

#[derive(Component, Deref, Debug)]
#[relationship_target(relationship = EventOf, linked_spawn)]
pub struct Events(Vec<Entity>);

/// Events of a track, ordered by start beat
pub type EventTrack = BTreeMap<(Beat, Entity), LineEvent>;

/// Events of every line, split by track and ordered by start beat
#[derive(Resource, Default)]
pub struct EventIndex(HashMap<(Entity, LineEventKind), EventTrack>);

impl EventIndex {
    pub fn track(&self, line: Entity, kind: LineEventKind) -> Option<&EventTrack> {
        self.0.get(&(line, kind))
    }

    /// Evaluate a track of a line at the given beat
    ///
    /// Equals the max of every event's inclusive evaluation as long as the events on the track do not overlap.
    /// Only the last two events starting at or before the beat matter then, since the earlier one may end right at it.
    pub fn evaluate(&self, line: Entity, kind: LineEventKind, beat: f32) -> EventEvaluationResult {
        let Some(track) = self.track(line, kind) else {
            return EventEvaluationResult::Unaffected;
        };
        // every event starting at or before the beat starts before this bound
        let bound = beat!(beat.floor() as i32 + 1);
        track
            .range(..(bound, Entity::PLACEHOLDER))
            .rev()
            .map(|(_, event)| event)
            .filter(|event| event.start_beat.value() <= beat)
            .take(2)
            .map(|event| event.evaluate_inclusive(beat))
            .max()
            .unwrap_or(EventEvaluationResult::Unaffected)
    }
}

pub struct EventIndexPlugin;

impl Plugin for EventIndexPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EventIndex>()
            .add_observer(index::<LineEvent>)
            .add_observer(index::<EventOf>)
            .add_observer(unindex::<LineEvent>)
            .add_observer(unindex::<EventOf>);
    }
}

// both components are immutable, so every change goes through these two triggers

fn index<C: Component>(
    trigger: On<Insert, C>,
    events: Query<(&LineEvent, &EventOf)>,
    mut index: ResMut<EventIndex>,
) {
    let Ok((event, event_of)) = events.get(trigger.entity) else {
        return;
    };
    index
        .0
        .entry((event_of.0, event.kind))
        .or_default()
        .insert((event.start_beat, trigger.entity), *event);
}

fn unindex<C: Component>(
    trigger: On<Replace, C>,
    events: Query<(&LineEvent, &EventOf)>,
    mut index: ResMut<EventIndex>,
) {
    let Ok((event, event_of)) = events.get(trigger.entity) else {
        return;
    };
    let key = (event_of.0, event.kind);
    if let Some(track) = index.0.get_mut(&key) {
        track.remove(&(event.start_beat, trigger.entity));
        if track.is_empty() {
            index.0.remove(&key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phichain_chart::beat;
    use phichain_chart::event::LineEventValue;

    type Snapshot = Vec<((Entity, u8), Vec<(Beat, Entity, LineEvent)>)>;

    fn event(kind: LineEventKind, start: i32) -> LineEvent {
        LineEvent {
            kind,
            start_beat: beat!(start),
            end_beat: beat!(start + 1),
            value: LineEventValue::constant(start as f32),
        }
    }

    fn indexed(world: &World) -> Snapshot {
        let mut tracks: Snapshot = world
            .resource::<EventIndex>()
            .0
            .iter()
            .map(|(&(line, kind), track)| {
                let events = track
                    .iter()
                    .map(|(&(beat, entity), &event)| (beat, entity, event))
                    .collect();
                ((line, kind.into()), events)
            })
            .collect();
        tracks.sort_by_key(|(key, _)| *key);
        tracks
    }

    fn rebuilt(world: &mut World) -> Snapshot {
        let mut tracks = BTreeMap::<(Entity, u8), Vec<_>>::new();
        for (entity, event, event_of) in world.query::<(Entity, &LineEvent, &EventOf)>().iter(world)
        {
            tracks
                .entry((event_of.0, event.kind.into()))
                .or_default()
                .push((event.start_beat, entity, *event));
        }
        for events in tracks.values_mut() {
            events.sort_by_key(|(beat, entity, _)| (*beat, *entity));
        }
        tracks.into_iter().collect()
    }

    #[test]
    fn evaluate_matches_every_event_when_events_do_not_overlap() {
        use phichain_chart::easing::Easing;
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};

        let mut rng = StdRng::seed_from_u64(0);
        let mut world = World::new();
        let line = world.spawn_empty().id();
        for _ in 0..200 {
            // in quarter beats, so seams and gaps land on exact beats
            let mut quarter = rng.random_range(-4..4);
            let mut track = EventTrack::new();
            for _ in 0..rng.random_range(0..12) {
                quarter += rng.random_range(0..3);
                let start = beat!(0, quarter, 4);
                quarter += rng.random_range(1..6);
                let end = beat!(0, quarter, 4);
                let value = if rng.random_bool(0.5) {
                    LineEventValue::constant(rng.random_range(-10.0..10.0))
                } else {
                    LineEventValue::transition(
                        rng.random_range(-10.0..10.0),
                        rng.random_range(-10.0..10.0),
                        Easing::EaseInOutSine,
                    )
                };
                let event = LineEvent {
                    kind: LineEventKind::X,
                    start_beat: start,
                    end_beat: end,
                    value,
                };
                track.insert((start, world.spawn_empty().id()), event);
            }

            let mut beats: Vec<f32> = (0..30)
                .map(|_| rng.random_range(-3.0..quarter as f32 / 4.0 + 2.0))
                .collect();
            beats.extend(
                track
                    .values()
                    .flat_map(|e| [e.start_beat.value(), e.end_beat.value()]),
            );

            let mut index = EventIndex::default();
            index.0.insert((line, LineEventKind::X), track.clone());
            for beat in beats {
                let every = track
                    .values()
                    .map(|event| event.evaluate_inclusive(beat))
                    .max()
                    .unwrap_or(EventEvaluationResult::Unaffected);
                assert_eq!(
                    index.evaluate(line, LineEventKind::X, beat),
                    every,
                    "at {beat}"
                );
            }
        }
    }

    #[test]
    fn index_follows_every_change() {
        let mut app = App::new();
        app.add_plugins(EventIndexPlugin);
        let world = app.world_mut();

        let a = world.spawn_empty().id();
        let b = world.spawn_empty().id();
        let moved = world.spawn((event(LineEventKind::X, 2), EventOf(a))).id();
        let retracked = world.spawn((event(LineEventKind::Y, 1), EventOf(a))).id();
        let relined = world.spawn((event(LineEventKind::X, 0), EventOf(a))).id();
        let despawned = world.spawn((event(LineEventKind::X, 5), EventOf(b))).id();
        world.spawn((event(LineEventKind::X, 4), EventOf(b)));

        world.entity_mut(moved).insert(event(LineEventKind::X, 7));
        world
            .entity_mut(retracked)
            .insert(event(LineEventKind::X, 3));
        world.entity_mut(relined).insert(EventOf(b));
        world.entity_mut(despawned).despawn();
        assert_eq!(indexed(world), rebuilt(world));

        // events are despawned along with their line
        world.entity_mut(a).despawn();
        assert_eq!(indexed(world), rebuilt(world));
        assert!(world
            .resource::<EventIndex>()
            .track(a, LineEventKind::X)
            .is_none());
    }
}
