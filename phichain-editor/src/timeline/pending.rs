use crate::notification::{ToastsExt, ToastsStorage};
use crate::schedule::EditorSet;
use crate::timeline::pending_status::PendingStatus;
use crate::timeline::target::TimelineTargets;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use phichain_game::Pending;

/// The timeline a [`Pending`] object is being placed on
#[derive(Component, Debug)]
#[relationship(relationship_target = TimelinePending)]
#[require(Pending)]
pub struct PendingOf(pub Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = PendingOf, linked_spawn)]
pub struct TimelinePending(Vec<Entity>);

impl TimelinePending {
    /// The pending object on the timeline, there is at most one
    pub fn entity(&self) -> Option<Entity> {
        self.0.first().copied()
    }
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
                cancel_undisplayed_pending
                    .run_if(any_with_component::<PendingOf>)
                    .before(EditorSet::Edit),
            ),
        );
    }
}

/// Start a pending object on the timeline `owner`, replacing the existing one
pub fn spawn_pending(
    commands: &mut Commands,
    owner: Entity,
    status: PendingStatus,
    bundle: impl Bundle,
) {
    commands.queue(move |world: &mut World| {
        if cancel_pending(world) {
            if let Some(mut toasts) = world.get_resource_mut::<ToastsStorage>() {
                notify_cancelled(&mut toasts);
            }
        }
        world.spawn((bundle, PendingOf(owner), status));
    });
}

/// A pending object only exists in its view, where its owner shows it
fn cancel_undisplayed_pending(
    mut commands: Commands,
    pending: Query<(Entity, &PendingOf)>,
    targets: TimelineTargets,
    toasts: Option<ResMut<ToastsStorage>>,
) {
    let mut cancelled = false;
    for (entity, owner) in &pending {
        if !targets.displays(owner.0, entity) {
            commands.entity(entity).despawn();
            cancelled = true;
        }
    }
    if let (true, Some(mut toasts)) = (cancelled, toasts) {
        notify_cancelled(&mut toasts);
    }
}

fn notify_cancelled(toasts: &mut ToastsStorage) {
    toasts.info(t!("tab.timeline.pending.cancelled"));
}

fn cancel_pending_on_escape(world: &mut World) {
    cancel_pending(world);
}

/// Despawn the pending object, returning whether there was one
fn cancel_pending(world: &mut World) -> bool {
    let pending: Vec<_> = world
        .query_filtered::<Entity, With<PendingOf>>()
        .iter(world)
        .collect();
    for &entity in &pending {
        world.despawn(entity);
    }
    !pending.is_empty()
}
