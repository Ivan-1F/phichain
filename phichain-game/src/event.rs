use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use phichain_chart::beat::Beat;
use phichain_chart::event::{LineEvent, LineEventKind};
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
