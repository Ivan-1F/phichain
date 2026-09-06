use crate::events::{EditorEvent, EditorEventAppExt};
use crate::utils::entity::replace_with_empty;
use bevy::app::{App, Plugin};
use bevy::log::debug;
use bevy::prelude::{ChildOf, Entity, Message, World};
use bon::Builder;
use phichain_chart::id::NoteId;
use phichain_chart::note::Note;

pub struct NoteEventPlugin;

impl Plugin for NoteEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_editor_event::<SpawnNoteEvent>()
            .add_editor_event::<DespawnNoteEvent>();
    }
}

#[derive(Debug, Clone, Message, Builder)]
pub struct SpawnNoteEvent {
    note: Note,
    id: NoteId,
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
            .insert((self.note, self.id, ChildOf(self.line_entity)))
            .id()
    }
}

#[derive(Debug, Clone, Message, Builder)]
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
            world.entity_mut(self.target).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id_index::{IdIndex, IdIndexPlugin};
    use phichain_chart::beat::Beat;
    use phichain_chart::line::Line;
    use phichain_chart::note::NoteKind;

    #[test]
    fn indexes_explicit_id_when_creating_and_restoring_a_note() {
        let mut app = App::new();
        app.add_plugins(IdIndexPlugin);
        let world = app.world_mut();
        let line = world.spawn(Line::default()).id();
        let id = NoteId::new();
        let note = Note::new(NoteKind::Tap, true, Beat::ONE, 100.0, 1.0);
        let mut target = None;

        for _ in 0..2 {
            let entity = SpawnNoteEvent::builder()
                .note(note)
                .id(id)
                .line_entity(line)
                .maybe_target(target)
                .build()
                .run(world);
            assert_eq!(world.get::<Note>(entity), Some(&note));
            assert_eq!(world.get::<NoteId>(entity), Some(&id));
            assert_eq!(world.resource::<IdIndex>().entity(id.uuid()), Some(entity));

            DespawnNoteEvent::builder()
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
