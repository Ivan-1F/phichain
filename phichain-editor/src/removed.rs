use bevy::app::{App, Plugin, PostUpdate};
use bevy::ecs::entity_disabling::Disabled;
use bevy::prelude::{
    Changed, Commands, Component, Entity, EntityWorldMut, Or, Query, RelationshipTarget, With,
    Without,
};

/// A reference-counted disabled marker
///
/// This acts as a reference counter. Each time the entity is "removed" (e.g., via a direct removal
/// or as a child of another removed entity), the counter is incremented.
///
/// Undoing a removal decrements the counter. When `count > 0`, the entity should be considered disabled.
/// When `count == 0`, it is considered active.
#[derive(Debug, Clone, Component)]
pub struct Removed {
    count: u32,
}

pub struct RemovedPlugin;

impl Plugin for RemovedPlugin {
    fn build(&self, app: &mut App) {
        // TODO: this assumes all editing (operations related to `Removed` component) happens in `Update`
        // TODO: use custom schedule for this
        app.add_systems(PostUpdate, apply_removed_system);
    }
}

/// Apply Bevy's [`Disabled`] marker according to the [`Removed`] component
///
/// - If `Removed.count > 0`, inserts the [`Disabled`] marker.
/// - If `Removed.count == 0`, removes both [`Disabled`] and [`Removed`].
pub fn apply_removed_system(
    mut commands: Commands,
    query: Query<
        (Entity, &Removed),
        (
            Or<(With<Disabled>, Without<Disabled>)>, // explicitly query for Disabled entities as well
            Changed<Removed>,
        ),
    >,
    disabled_query: Query<(), With<Disabled>>,
) {
    for (entity, removed) in &query {
        if removed.count > 0 {
            if disabled_query.get(entity).is_err() {
                commands.entity(entity).insert(Disabled);
            }
        } else {
            commands.entity(entity).remove::<Disabled>();
            commands.entity(entity).remove::<Removed>();
        }
    }
}

pub trait RemovedExt {
    /// Increase the [`Removed`] counter for the entity and all related entities,
    /// traversing the relationship tracked in `S` in a breadth-first manner.
    ///
    /// # Warning
    ///
    /// This method should only be called on relationships that form a tree-like structure.
    /// Any cycles will cause this method to loop infinitely.
    fn increase_removed<S: RelationshipTarget>(&mut self) -> &mut Self;

    /// Decrease the [`Removed`] counter for the entity and all related entities,
    /// traversing the relationship tracked in `S` in a breadth-first manner.
    ///
    /// # Warning
    ///
    /// This method should only be called on relationships that form a tree-like structure.
    /// Any cycles will cause this method to loop infinitely.
    fn decrease_removed<S: RelationshipTarget>(&mut self) -> &mut Self;
}

impl RemovedExt for EntityWorldMut<'_> {
    fn increase_removed<S: RelationshipTarget>(&mut self) -> &mut Self {
        match self.get_mut::<Removed>() {
            None => {
                self.insert(Removed { count: 1 });
            }
            Some(mut removed) => {
                removed.as_mut().count += 1;
            }
        }

        if let Some(relationship_target) = self.get::<S>() {
            let related_vec: Vec<Entity> = relationship_target.iter().collect();
            for related in related_vec {
                self.world_scope(|world| {
                    world.entity_mut(related).increase_removed::<S>();
                });
            }
        }

        self
    }

    fn decrease_removed<S: RelationshipTarget>(&mut self) -> &mut Self {
        if let Some(mut removed) = self.get_mut::<Removed>() {
            removed.as_mut().count -= 1;
        }

        if let Some(relationship_target) = self.get::<S>() {
            let related_vec: Vec<Entity> = relationship_target.iter().collect();
            for related in related_vec {
                self.world_scope(|world| {
                    world.entity_mut(related).decrease_removed::<S>();
                });
            }
        }

        self
    }
}
