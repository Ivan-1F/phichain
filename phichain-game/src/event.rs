use bevy::prelude::*;

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
