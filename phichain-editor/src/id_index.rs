use bevy::ecs::component::Immutable;
use bevy::ecs::entity::EntityHashMap;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use phichain_chart::id::{
    BpmPointId, CurveNoteTrackId, EventId, LineId, NoteId, ObjectId, ProjectId,
};
use uuid::Uuid;

/// Lookup between chart object IDs and their current ECS entities.
#[derive(Resource, Default)]
pub struct IdIndex {
    by_id: HashMap<Uuid, Entity>,
    by_entity: EntityHashMap<Uuid>,
}

impl IdIndex {
    pub fn entity(&self, id: Uuid) -> Option<Entity> {
        self.by_id.get(&id).copied()
    }

    pub fn id(&self, entity: Entity) -> Option<Uuid> {
        self.by_entity.get(&entity).copied()
    }
}

pub struct IdIndexPlugin;

impl Plugin for IdIndexPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IdIndex>();
        register::<NoteId>(app);
        register::<EventId>(app);
        register::<LineId>(app);
        register::<CurveNoteTrackId>(app);
        register::<BpmPointId>(app);
        register::<ProjectId>(app);
    }
}

fn register<T: Component<Mutability = Immutable> + Copy + Into<ObjectId>>(app: &mut App) {
    app.world_mut()
        .register_component_hooks::<T>()
        .on_add(add::<T>)
        .on_insert(check_id::<T>)
        .on_remove(remove);
}

fn add<T: Component + Copy + Into<ObjectId>>(mut world: DeferredWorld, ctx: HookContext) {
    let id: ObjectId = (*world.get::<T>(ctx.entity).unwrap()).into();
    let id = id.uuid();
    let mut index = world.resource_mut::<IdIndex>();
    assert!(
        index.id(ctx.entity).is_none(),
        "an entity can only have one object ID"
    );
    assert!(index.entity(id).is_none(), "duplicate object ID {id}");
    // Add hooks run before Add observers, so the index is already available there.
    index.by_id.insert(id, ctx.entity);
    index.by_entity.insert(ctx.entity, id);
}

fn check_id<T: Component + Copy + Into<ObjectId>>(world: DeferredWorld, ctx: HookContext) {
    let id: ObjectId = (*world.get::<T>(ctx.entity).unwrap()).into();
    // Immutable components can still be replaced with insert.
    assert_eq!(
        world.resource::<IdIndex>().id(ctx.entity),
        Some(id.uuid()),
        "object IDs cannot change"
    );
}

fn remove(mut world: DeferredWorld, ctx: HookContext) {
    let mut index = world.resource_mut::<IdIndex>();
    if let Some(id) = index.by_entity.remove(&ctx.entity) {
        index.by_id.remove(&id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phichain_chart::beat::Beat;
    use phichain_chart::note::{Note, NoteKind};

    #[test]
    fn indexes_all_id_types_and_cleans_up_removed_components_and_entities() {
        let mut app = App::new();
        app.add_plugins(IdIndexPlugin);
        let note = NoteId::new();
        let event = EventId::new();
        let line = LineId::new();
        let track = CurveNoteTrackId::new();
        let bpm = BpmPointId::new();
        let project = ProjectId::new();
        let objects = [
            (app.world_mut().spawn(note).id(), note.uuid()),
            (app.world_mut().spawn(event).id(), event.uuid()),
            (app.world_mut().spawn(line).id(), line.uuid()),
            (app.world_mut().spawn(track).id(), track.uuid()),
            (app.world_mut().spawn(bpm).id(), bpm.uuid()),
            (app.world_mut().spawn(project).id(), project.uuid()),
        ];
        for (entity, id) in objects {
            let index = app.world().resource::<IdIndex>();
            assert_eq!(index.entity(id), Some(entity));
            assert_eq!(index.id(entity), Some(id));
        }
        app.world_mut().entity_mut(objects[0].0).insert(note);
        app.world_mut().entity_mut(objects[0].0).remove::<NoteId>();
        assert!(app
            .world()
            .resource::<IdIndex>()
            .entity(note.uuid())
            .is_none());
        assert!(app.world().resource::<IdIndex>().id(objects[0].0).is_none());
        for (entity, id) in objects {
            app.world_mut().entity_mut(entity).despawn();
            let index = app.world().resource::<IdIndex>();
            assert!(index.entity(id).is_none());
            assert!(index.id(entity).is_none());
        }
        let restored = app.world_mut().spawn(note).id();
        assert_ne!(restored, objects[0].0);
        assert_eq!(
            app.world().resource::<IdIndex>().entity(note.uuid()),
            Some(restored)
        );
    }

    #[test]
    fn index_is_available_to_component_lifecycle_observers() {
        let mut app = App::new();
        app.add_plugins(IdIndexPlugin)
            .add_observer(
                |event: On<Add, Note>, ids: Query<&NoteId>, index: Res<IdIndex>| {
                    assert_eq!(
                        index.id(event.entity),
                        Some(ids.get(event.entity).unwrap().uuid())
                    );
                },
            )
            .add_observer(
                |event: On<Replace, NoteId>, ids: Query<&NoteId>, index: Res<IdIndex>| {
                    assert_eq!(
                        index.id(event.entity),
                        Some(ids.get(event.entity).unwrap().uuid())
                    );
                },
            )
            .add_observer(
                |event: On<Remove, NoteId>, ids: Query<&NoteId>, index: Res<IdIndex>| {
                    assert_eq!(
                        index.id(event.entity),
                        Some(ids.get(event.entity).unwrap().uuid())
                    );
                },
            );
        let entity = app
            .world_mut()
            .spawn(Note::new(NoteKind::Tap, true, Beat::ONE, 0.0, 1.0))
            .id();
        app.world_mut().entity_mut(entity).despawn();
        assert!(app.world().resource::<IdIndex>().id(entity).is_none());
    }

    #[test]
    #[should_panic(expected = "duplicate object ID")]
    fn duplicate_ids_are_rejected_across_object_types() {
        let mut app = App::new();
        app.add_plugins(IdIndexPlugin);
        let id = NoteId::new();
        app.world_mut().spawn(id);
        app.world_mut().spawn(EventId::from_uuid(id.uuid()));
    }

    #[test]
    #[should_panic(expected = "an entity can only have one object ID")]
    fn an_entity_cannot_have_multiple_ids() {
        let mut app = App::new();
        app.add_plugins(IdIndexPlugin);
        app.world_mut().spawn((NoteId::new(), EventId::new()));
    }

    #[test]
    #[should_panic(expected = "object IDs cannot change")]
    fn id_replacement_is_rejected() {
        let mut app = App::new();
        app.add_plugins(IdIndexPlugin);
        let entity = app.world_mut().spawn(NoteId::new()).id();
        app.world_mut().entity_mut(entity).insert(NoteId::new());
    }
}
