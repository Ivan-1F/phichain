use crate::events::{EditorEvent, EditorEventAppExt};
use crate::utils::entity::replace_with_empty;
use bevy::prelude::*;
use bon::Builder;
use phichain_chart::event::{LineEvent, LineEventBundle};

pub struct LineEventEventPlugin;

impl Plugin for LineEventEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_editor_event::<SpawnLineEventEvent>()
            .add_editor_event::<DespawnLineEventEvent>();
    }
}

#[derive(Debug, Clone, Event, Builder)]
pub struct SpawnLineEventEvent {
    event: LineEvent,
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
            .insert(LineEventBundle::new(self.event))
            .set_parent(self.line_entity)
            .id()
    }
}

#[derive(Debug, Clone, Event, Builder)]
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
            world.entity_mut(self.target).despawn_recursive();
        }
    }
}
