use bevy::prelude::{Children, DespawnRecursiveExt, Entity, World};

/// Replace the given entity with an empty one. Removes all its children and components
pub fn replace_with_empty(world: &mut World, entity: Entity) {
    // despawn all children
    if let Some(children) = world.entity_mut(entity).take::<Children>() {
        for child in children.iter() {
            world.entity_mut(*child).despawn_recursive();
        }
    }

    // remove all components
    world.entity_mut(entity).retain::<()>();
}
