use bevy::prelude::{Component, Deref, Entity};

#[derive(Component)]
#[relationship(relationship_target = Events)]
pub struct EventOf(pub Entity);

impl EventOf {
    /// The target entity of this event entity.
    #[inline]
    pub fn target(&self) -> Entity {
        self.0
    }
}

#[derive(Component, Deref)]
#[relationship_target(relationship = EventOf, linked_spawn)]
pub struct Events(Vec<Entity>);
