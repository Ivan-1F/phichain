use crate::events::{EditorEvent, EditorEventAppExt};
use crate::utils::entity::replace_with_empty;
use bevy::prelude::*;
use bon::Builder;
use phichain_chart::event::LineEvent;
use phichain_chart::id::EventId;
use phichain_game::event::EventOf;

pub struct LineEventEventPlugin;

impl Plugin for LineEventEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_editor_event::<SpawnLineEventEvent>()
            .add_editor_event::<DespawnLineEventEvent>();
    }
}

#[derive(Debug, Clone, Message, Builder)]
pub struct SpawnLineEventEvent {
    event: LineEvent,
    id: EventId,
    line_entity: Entity,
    target: Option<Entity>,
}

impl EditorEvent for SpawnLineEventEvent {
    type Output = Entity;

    fn run(self, world: &mut World) -> Self::Output {
        match self.target {
            None => {
                debug!("spawned event {:?} on new entity", self.event);
            }
            Some(target) => {
                debug!("spawned event {:?} on entity {:?}", self.event, target);
            }
        }
        let id = match self.target {
            None => world.spawn_empty().id(),
            Some(target) => target,
        };
        world
            .entity_mut(id)
            .insert((self.event, self.id, EventOf(self.line_entity)))
            .id()
    }
}

#[derive(Debug, Clone, Message, Builder)]
pub struct DespawnLineEventEvent {
    target: Entity,
    #[builder(default = false)]
    keep_entity: bool,
}

impl EditorEvent for DespawnLineEventEvent {
    type Output = ();

    fn run(self, world: &mut World) -> Self::Output {
        debug!(
            "despawned event {:?}{}",
            self.target,
            if self.keep_entity {
                " (keep entity)"
            } else {
                ""
            }
        );
        if self.keep_entity {
            replace_with_empty(world, self.target);
        } else {
            world.entity_mut(self.target).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id_index::{IdIndex, IdIndexPlugin};
    use phichain_chart::beat::Beat;
    use phichain_chart::event::{LineEventKind, LineEventValue};
    use phichain_chart::line::Line;

    #[test]
    fn indexes_explicit_id_when_creating_and_restoring_an_event() {
        let mut app = App::new();
        app.add_plugins(IdIndexPlugin);
        let world = app.world_mut();
        let line = world.spawn(Line::default()).id();
        let id = EventId::new();
        let event = LineEvent {
            kind: LineEventKind::X,
            start_beat: Beat::ZERO,
            end_beat: Beat::ONE,
            value: LineEventValue::constant(100.0),
        };
        let mut target = None;

        for _ in 0..2 {
            let entity = SpawnLineEventEvent::builder()
                .event(event)
                .id(id)
                .line_entity(line)
                .maybe_target(target)
                .build()
                .run(world);
            assert_eq!(world.get::<LineEvent>(entity), Some(&event));
            assert_eq!(world.get::<EventId>(entity), Some(&id));
            assert_eq!(world.resource::<IdIndex>().entity(id.uuid()), Some(entity));

            DespawnLineEventEvent::builder()
                .target(entity)
                .keep_entity(true)
                .build()
                .run(world);
            assert!(world.resource::<IdIndex>().entity(id.uuid()).is_none());
            assert!(world.resource::<IdIndex>().id(entity).is_none());
            target = Some(entity);
        }
    }
}
