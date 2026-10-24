use crate::editing::description::ObjectCounts;
use crate::id_index::IdIndex;
use bevy::ecs::component::Immutable;
use bevy::ecs::entity::EntityMapper;
use bevy::ecs::reflect::{AppTypeRegistry, ReflectComponent};
use bevy::prelude::*;
use bevy::reflect::{FromType, GetTypeRegistration, TypePath};
use phichain_chart::event::LineEvent;
use phichain_chart::note::Note;
use phichain_game::curve_note_track::CurveNoteTrackTo;
use phichain_game::{Derived, Pending};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

type Value = Box<dyn Reflect>;

#[derive(Resource, Default)]
pub(super) struct Registry {
    types: HashMap<TypeId, TrackedType>,
}

struct TrackedType {
    component: ReflectComponent,
    identity: bool,
}

pub(super) fn register_component<T>(app: &mut App, identity: bool)
where
    T: Component<Mutability = Immutable>
        + Reflect
        + FromReflect
        + GetTypeRegistration
        + TypePath
        + Clone,
{
    app.register_type::<T>();
    let old = app.world_mut().resource_mut::<Registry>().types.insert(
        TypeId::of::<T>(),
        TrackedType {
            component: <ReflectComponent as FromType<T>>::from_type(),
            identity,
        },
    );
    assert!(old.is_none(), "document component registered twice");
    app.add_observer(record_add::<T>)
        .add_observer(record_replace::<T>);
}

#[derive(Resource, Default)]
pub(super) struct Recorder {
    pub enabled: bool,
    replaying: bool,
    active: Option<PendingChanges>,
}

impl Recorder {
    pub fn begin(&mut self) {
        self.resume(PendingChanges::default());
    }

    pub fn resume(&mut self, pending: PendingChanges) {
        assert!(self.active.is_none(), "nested document edit");
        self.active = Some(pending);
    }

    pub fn finish(&mut self) -> PendingChanges {
        self.active.take().expect("no active document edit")
    }
}

#[derive(Default)]
pub(super) struct PendingChanges {
    before: HashMap<(Uuid, TypeId), Option<Value>>,
    touched: HashMap<Uuid, Entity>,
    born: HashSet<Uuid>,
    references: HashMap<Entity, Uuid>,
}

fn record_add<T: Component + Reflect>(
    event: On<Add, T>,
    query: Query<EntityRef>,
    registry: Res<Registry>,
    index: Res<IdIndex>,
    mut recorder: ResMut<Recorder>,
) {
    capture::<T>(
        query.get(event.entity).unwrap(),
        None,
        &registry,
        &index,
        &mut recorder,
    );
}

fn record_replace<T: Component + Reflect + Clone>(
    event: On<Replace, T>,
    query: Query<EntityRef>,
    registry: Res<Registry>,
    index: Res<IdIndex>,
    mut recorder: ResMut<Recorder>,
) {
    let entity = query.get(event.entity).unwrap();
    // Clone only the first old value, not every intermediate drag update.
    if !recorder.enabled
        || recorder.replaying
        || entity.contains::<Derived>()
        || entity.contains::<Pending>()
    {
        return;
    }
    if let Some(id) = index.id(entity.id()) {
        if recorder
            .active
            .as_ref()
            .is_some_and(|pending| pending.before.contains_key(&(id, TypeId::of::<T>())))
        {
            return;
        }
    }
    let before = entity
        .get::<T>()
        .map(|value| Box::new(value.clone()) as Value);
    capture::<T>(entity, before, &registry, &index, &mut recorder);
}

fn capture<T: Component + Reflect>(
    entity: EntityRef,
    mut before: Option<Value>,
    registry: &Registry,
    index: &IdIndex,
    recorder: &mut Recorder,
) {
    if !recorder.enabled
        || recorder.replaying
        || entity.contains::<Derived>()
        || entity.contains::<Pending>()
    {
        return;
    }
    let Some(id) = index.id(entity.id()) else {
        return;
    };
    let pending = recorder
        .active
        .as_mut()
        .expect("document mutation outside Edits");
    let ty = TypeId::of::<T>();
    pending.touched.insert(id, entity.id());
    pending.references.insert(entity.id(), id);
    let descriptor = &registry.types[&ty];
    if descriptor.identity && before.is_none() && !pending.before.contains_key(&(id, ty)) {
        pending.born.insert(id);
    }
    if pending.before.contains_key(&(id, ty)) {
        return;
    }
    if let Some(value) = &mut before {
        collect_references(
            value.as_mut(),
            &descriptor.component,
            index,
            &mut pending.references,
        );
    }
    pending.before.insert((id, ty), before);
}

struct ReferenceCollector<'a> {
    index: &'a IdIndex,
    references: &'a mut HashMap<Entity, Uuid>,
}

impl EntityMapper for ReferenceCollector<'_> {
    fn get_mapped(&mut self, entity: Entity) -> Entity {
        let id = self
            .index
            .id(entity)
            .or_else(|| self.references.get(&entity).copied())
            .expect("document component references an object without a registered identity");
        self.references.insert(entity, id);
        entity
    }

    fn set_mapped(&mut self, _: Entity, _: Entity) {
        unreachable!("component mapping must only visit references")
    }
}

fn collect_references(
    value: &mut dyn Reflect,
    component: &ReflectComponent,
    index: &IdIndex,
    references: &mut HashMap<Entity, Uuid>,
) {
    component.map_entities(value, &mut ReferenceCollector { index, references });
}

fn clone_value(value: &dyn Reflect) -> Value {
    value
        .reflect_clone()
        .expect("document components must support ReflectClone")
}

struct ComponentChange {
    ty: TypeId,
    before: Option<Value>,
    after: Option<Value>,
}

struct ObjectChange {
    id: Uuid,
    existed_before: bool,
    exists_after: bool,
    components: Vec<ComponentChange>,
}

pub struct ChangeSet {
    objects: Vec<ObjectChange>,
    references: HashMap<Entity, Uuid>,
}

impl PendingChanges {
    pub fn is_empty(&self) -> bool {
        self.before.is_empty()
    }

    pub fn settle(mut self, world: &World) -> ChangeSet {
        let registry = world.resource::<Registry>();
        let index = world.resource::<IdIndex>();
        // An identity may be inserted after some of the object's components.
        // Capture the complete initial state once the object has been assembled.
        for id in &self.born {
            if let Ok(entity) = world.get_entity(self.touched[id]) {
                for (ty, descriptor) in &registry.types {
                    if descriptor.component.reflect(entity).is_some() {
                        self.before.entry((*id, *ty)).or_insert(None);
                    }
                }
            }
        }
        let mut objects: HashMap<Uuid, ObjectChange> = HashMap::new();
        for ((id, ty), mut before) in self.before {
            let existed_before = !self.born.contains(&id);
            let entity = world.get_entity(self.touched[&id]).ok();
            if let Some(entity) = entity {
                assert_eq!(
                    index.id(entity.id()),
                    Some(id),
                    "despawn document objects instead of removing their identity"
                );
            }
            let exists_after = entity.is_some();
            if !existed_before {
                before = None;
            }
            let descriptor = &registry.types[&ty];
            let mut after = entity
                .and_then(|entity| descriptor.component.reflect(entity))
                .map(clone_value);
            if let Some(value) = &mut after {
                collect_references(
                    value.as_mut(),
                    &descriptor.component,
                    index,
                    &mut self.references,
                );
            }
            let equal = match (&before, &after) {
                (None, None) => true,
                (Some(a), Some(b)) => a
                    .reflect_partial_eq(b.as_partial_reflect())
                    .expect("document components must support reflected equality"),
                _ => false,
            };
            if equal {
                continue;
            }
            objects
                .entry(id)
                .or_insert_with(|| ObjectChange {
                    id,
                    existed_before,
                    exists_after,
                    components: Vec::new(),
                })
                .components
                .push(ComponentChange { ty, before, after });
        }
        ChangeSet {
            objects: objects.into_values().collect(),
            references: self.references,
        }
    }
}

struct ReplayMapper<'a> {
    references: &'a HashMap<Entity, Uuid>,
    entities: &'a HashMap<Uuid, Entity>,
}

impl EntityMapper for ReplayMapper<'_> {
    fn get_mapped(&mut self, entity: Entity) -> Entity {
        self.entities[&self.references[&entity]]
    }
    fn set_mapped(&mut self, _: Entity, _: Entity) {
        unreachable!("component mapping must only visit references")
    }
}

impl ChangeSet {
    pub(super) fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    /// Objects this edit created, classified for history labels.
    pub fn created_counts(&self) -> ObjectCounts {
        self.counts(|object| !object.existed_before && object.exists_after)
    }

    /// Objects this edit destroyed, classified for history labels. Cascaded
    /// deletions are recorded like any other removal, so they are included.
    pub fn destroyed_counts(&self) -> ObjectCounts {
        self.counts(|object| object.existed_before && !object.exists_after)
    }

    fn counts(&self, predicate: impl Fn(&ObjectChange) -> bool) -> ObjectCounts {
        let mut counts = ObjectCounts::default();
        for object in self.objects.iter().filter(|object| predicate(object)) {
            for change in &object.components {
                counts.notes += usize::from(change.ty == TypeId::of::<Note>());
                counts.events += usize::from(change.ty == TypeId::of::<LineEvent>());
                counts.tracks += usize::from(change.ty == TypeId::of::<CurveNoteTrackTo>());
            }
        }
        counts
    }

    pub(super) fn apply(&self, world: &mut World, after: bool) {
        world.resource_mut::<Recorder>().replaying = true;
        let index = world.resource::<IdIndex>();
        let mut entities: HashMap<_, _> = self
            .objects
            .iter()
            .map(|object| object.id)
            .chain(self.references.values().copied())
            .filter_map(|id| index.entity(id).map(|entity| (id, entity)))
            .collect();
        // Allocate all restored entities before mapping any references.
        for object in &self.objects {
            let alive = if after {
                object.exists_after
            } else {
                object.existed_before
            };
            if alive && !entities.contains_key(&object.id) {
                entities.insert(object.id, world.spawn_empty().id());
            }
        }
        let type_registry = world.resource::<AppTypeRegistry>().clone();
        let type_registry = type_registry.read();
        // Identity components precede data with required identity components.
        // Bevy still runs relationship hooks to maintain inverse collections.
        for identity in [true, false] {
            for object in &self.objects {
                if !(if after {
                    object.exists_after
                } else {
                    object.existed_before
                }) {
                    continue;
                }
                let entity = entities[&object.id];
                for change in &object.components {
                    let descriptor = &world.resource::<Registry>().types[&change.ty];
                    if descriptor.identity != identity {
                        continue;
                    }
                    let component = descriptor.component.clone();
                    if let Some(value) = if after { &change.after } else { &change.before } {
                        let mut value = clone_value(value.as_ref());
                        component.map_entities(
                            value.as_mut(),
                            &mut ReplayMapper {
                                references: &self.references,
                                entities: &entities,
                            },
                        );
                        component.insert(
                            &mut world.entity_mut(entity),
                            value.as_partial_reflect(),
                            &type_registry,
                        );
                    } else {
                        component.remove(&mut world.entity_mut(entity));
                    }
                }
            }
        }
        for object in &self.objects {
            if !(if after {
                object.exists_after
            } else {
                object.existed_before
            }) {
                if let Ok(entity) = world.get_entity_mut(entities[&object.id]) {
                    entity.despawn();
                }
            }
        }
        world.flush();
        world.resource_mut::<Recorder>().replaying = false;
    }
}
