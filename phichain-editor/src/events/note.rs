use crate::events::{EditorEvent, EditorEventAppExt};
use crate::utils::entity::replace_with_empty;
use bevy::app::{App, Plugin};
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::log::debug;
use bevy::prelude::{BuildChildren, Entity, Event, World};
use bon::Builder;
use phichain_chart::note::{Note, NoteBundle};

pub struct NoteEventPlugin;

impl Plugin for NoteEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_editor_event::<SpawnNoteEvent>()
            .add_editor_event::<DespawnNoteEvent>();
    }
}

#[derive(Debug, Clone, Event, Builder)]
pub struct SpawnNoteEvent {
    note: Note,
    line_entity: Entity,
    target: Option<Entity>,
}

impl EditorEvent for SpawnNoteEvent {
    type Output = Entity;

    fn run(self, world: &mut World) -> Self::Output {
        match self.target {
            None => {
                debug!("spawned note {:?} on new entity", self.note);
            }
            Some(target) => {
                debug!("spawned note {:?} on entity {:?}", self.note, target);
            }
        }
        let id = match self.target {
            None => world.spawn_empty().id(),
            Some(target) => target,
        };
        world
            .entity_mut(id)
            .insert(NoteBundle::new(self.note))
            .set_parent(self.line_entity)
            .id()
    }
}

#[derive(Debug, Clone, Event, Builder)]
pub struct DespawnNoteEvent {
    target: Entity,
    #[builder(default = false)]
    keep_entity: bool,
}

impl EditorEvent for DespawnNoteEvent {
    type Output = ();

    fn run(self, world: &mut World) -> Self::Output {
        debug!(
            "despawned note {:?}{}",
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
