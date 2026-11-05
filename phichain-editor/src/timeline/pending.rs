use bevy::prelude::*;

/// The timeline a [`phichain_game::Pending`] object is being placed on
#[derive(Component, Debug)]
#[relationship(relationship_target = TimelinePending)]
pub struct PendingOf(pub Entity);

#[derive(Component, Deref, Debug)]
#[relationship_target(relationship = PendingOf, linked_spawn)]
pub struct TimelinePending(Vec<Entity>);
