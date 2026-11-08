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

    /// Events of a track starting at or before the beat, latest first
    pub fn until(
        &self,
        line: Entity,
        kind: LineEventKind,
        beat: f32,
    ) -> impl Iterator<Item = (Entity, &LineEvent)> {
        // every event starting at or before the beat is ordered before this bound
        let bound = (beat!(beat.floor() as i32 + 1), Entity::PLACEHOLDER);
        self.track(line, kind)
            .into_iter()
            .flat_map(move |track| track.range(..bound).rev())
            .filter(move |(_, event)| event.start_beat.value() <= beat)
            .map(|(&(_, entity), event)| (entity, event))
    }

    /// Events of a track starting at or after the beat, earliest first
    pub fn since(
        &self,
        line: Entity,
        kind: LineEventKind,
        beat: f32,
    ) -> impl Iterator<Item = (Entity, &LineEvent)> {
        // every event starting at or after the beat is ordered after this bound
        let bound = (beat!(beat.floor() as i32 - 1), Entity::PLACEHOLDER);
        self.track(line, kind)
            .into_iter()
            .flat_map(move |track| track.range(bound..))
            .filter(move |(_, event)| event.start_beat.value() >= beat)
            .map(|(&(_, entity), event)| (entity, event))
    }

    /// Evaluate a track of a line at the given beat
    ///
    /// Equals the max of every event's inclusive evaluation as long as the events on the track do not overlap.
    /// Only the last two events starting at or before the beat matter then, since the earlier one may end right at it.
    pub fn evaluate(&self, line: Entity, kind: LineEventKind, beat: f32) -> EventEvaluationResult {
        self.until(line, kind, beat)
            .take(2)
            .map(|(_, event)| event.evaluate_inclusive(beat))
            .max()
            .unwrap_or(EventEvaluationResult::Unaffected)
    }

    /// Whether the event has length and shares no beat with its neighbors on the track, leaving out events `skip` accepts
    ///
    /// Checking the neighbors covers the whole track as long as the other events do not overlap.
    pub fn fits(
        &self,
        line: Entity,
        entity: Entity,
        event: &LineEvent,
        skip: impl Fn(Entity) -> bool,
    ) -> bool {
        let start = event.start_beat.value();
        let other = |(other, _): &(Entity, &LineEvent)| *other != entity && !skip(*other);
        event.start_beat < event.end_beat
            && self
                .until(line, event.kind, start)
                .find(other)
                .is_none_or(|(_, previous)| previous.end_beat <= event.start_beat)
            && self
                .since(line, event.kind, start)
                .find(other)
                .is_none_or(|(_, next)| event.end_beat <= next.start_beat)
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
    fn queries_match_scanning_every_event() {
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
                let until: Vec<_> = index
                    .until(line, LineEventKind::X, beat)
                    .map(|(e, _)| e)
                    .collect();
                let mut expected: Vec<_> = track
                    .iter()
                    .filter(|(_, event)| event.start_beat.value() <= beat)
                    .map(|(&(_, e), _)| e)
                    .collect();
                expected.reverse();
                assert_eq!(until, expected, "until {beat}");

                let since: Vec<_> = index
                    .since(line, LineEventKind::X, beat)
                    .map(|(e, _)| e)
                    .collect();
                let expected: Vec<_> = track
                    .iter()
                    .filter(|(_, event)| event.start_beat.value() >= beat)
                    .map(|(&(_, e), _)| e)
                    .collect();
                assert_eq!(since, expected, "since {beat}");

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
    fn fits_matches_comparing_every_event() {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};
        use std::collections::HashSet;

        let mut rng = StdRng::seed_from_u64(1);
        let mut world = World::new();
        let line = world.spawn_empty().id();
        let event = |start, end| LineEvent {
            kind: LineEventKind::X,
            start_beat: beat!(0, start, 4),
            end_beat: beat!(0, end, 4),
            value: LineEventValue::constant(0.0),
        };
        for _ in 0..500 {
            // the other events never overlap, in quarter beats
            let mut track = EventTrack::new();
            let mut quarter = 0;
            for _ in 0..rng.random_range(0..8) {
                quarter += rng.random_range(0..3);
                let start = quarter;
                quarter += rng.random_range(1..4);
                let other = event(start, quarter);
                track.insert((other.start_beat, world.spawn_empty().id()), other);
            }
            let start = rng.random_range(-2..quarter + 2);
            let candidate = event(start, start + rng.random_range(-1..6));
            let entity = world.spawn_empty().id();
            track.insert((candidate.start_beat, entity), candidate);
            let skipped: HashSet<Entity> = track
                .keys()
                .map(|&(_, other)| other)
                .filter(|_| rng.random_bool(0.2))
                .collect();

            let expected = candidate.start_beat < candidate.end_beat
                && track
                    .iter()
                    .filter(|((_, other), _)| *other != entity && !skipped.contains(other))
                    .all(|(_, other)| {
                        other.end_beat <= candidate.start_beat
                            || candidate.end_beat <= other.start_beat
                    });
            let mut index = EventIndex::default();
            index.0.insert((line, LineEventKind::X), track.clone());
            let fits = index.fits(line, entity, &candidate, |other| skipped.contains(&other));
            assert_eq!(fits, expected, "{candidate:?} in {track:?}");
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
