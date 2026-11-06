use crate::notification::{ToastsExt, ToastsStorage};
use crate::schedule::EditorSet;
use crate::selection::SelectedLine;
use crate::timeline::target::FollowSelectedLine;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use phichain_game::Pending;

/// The timeline a [`Pending`] object is being placed on
#[derive(Component, Debug)]
#[relationship(relationship_target = TimelinePending)]
pub struct PendingOf(pub Entity);

#[derive(Component, Deref, Debug)]
#[relationship_target(relationship = PendingOf, linked_spawn)]
pub struct TimelinePending(Vec<Entity>);

/// Whether the user cancelled a pending object themselves, implicit cancellations are notified
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cancellation {
    Explicit,
    Implicit,
}

pub struct PendingPlugin;

impl Plugin for PendingPlugin {
    fn build(&self, app: &mut App) {
        // Esc takes priority over placements pressed in the same frame
        app.configure_sets(
            Update,
            EditorSet::Edit.run_if(not(input_just_pressed(KeyCode::Escape))),
        )
        .add_systems(
            Update,
            (
                cancel_pending_on_escape.run_if(input_just_pressed(KeyCode::Escape)),
                cancel_pending_on_line_switch,
            ),
        );
    }
}

/// Start a pending object on the timeline `owner`, replacing the existing one
pub fn spawn_pending(commands: &mut Commands, owner: Entity, bundle: impl Bundle) {
    commands.queue(move |world: &mut World| {
        cancel_pending(world, Cancellation::Implicit);
        world.spawn((bundle, Pending, PendingOf(owner)));
    });
}

pub fn cancel_pending(world: &mut World, cancellation: Cancellation) {
    let pending = world
        .query_filtered::<Entity, With<PendingOf>>()
        .iter(world)
        .collect();
    cancel(world, pending, cancellation);
}

fn cancel(world: &mut World, pending: Vec<Entity>, cancellation: Cancellation) {
    if pending.is_empty() {
        return;
    }
    for entity in pending {
        world.despawn(entity);
    }
    if cancellation == Cancellation::Implicit {
        if let Some(mut toasts) = world.get_resource_mut::<ToastsStorage>() {
            toasts.info(t!("tab.timeline.pending.cancelled"));
        }
    }
}

fn cancel_pending_on_escape(world: &mut World) {
    cancel_pending(world, Cancellation::Explicit);
}

fn cancel_pending_on_line_switch(world: &mut World, mut last: Local<Option<Entity>>) {
    let selected = world.get_resource::<SelectedLine>().map(|line| line.0);
    let switched = last.is_some() && selected.is_some() && *last != selected;
    *last = selected;
    if !switched {
        return;
    }
    let mut pending = world.query::<(Entity, &PendingOf)>();
    let stale = pending
        .iter(world)
        .filter(|(_, owner)| world.get::<FollowSelectedLine>(owner.0).is_some())
        .map(|(entity, _)| entity)
        .collect();
    cancel(world, stale, Cancellation::Implicit);
}
