use crate::notification::{ToastsExt, ToastsStorage};
use crate::schedule::EditorSet;
use crate::timeline::pending_status::PendingStatus;
use crate::timeline::target::TimelineTargets;
use bevy::ecs::system::SystemParam;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use phichain_game::Pending;

/// The timeline a [`Pending`] object is being placed on
#[derive(Component, Debug)]
#[relationship(relationship_target = TimelinePending)]
#[require(Pending)]
pub struct PendingOf(pub Entity);

#[derive(Component, Deref, Debug)]
#[relationship_target(relationship = PendingOf, linked_spawn)]
pub struct TimelinePending(Vec<Entity>);

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
            cancel_pending_on_escape.run_if(input_just_pressed(KeyCode::Escape)),
        );
    }
}

/// The pending object the user can interact with on a timeline
#[derive(SystemParam)]
pub struct ActivePending<'w, 's> {
    pending: Query<'w, 's, (Entity, &'static PendingOf)>,
    targets: TimelineTargets<'w, 's>,
}

impl ActivePending<'_, '_> {
    /// The pending object owned by the timeline and placed on what it shows
    pub fn on(&self, timeline: Entity) -> Option<Entity> {
        self.pending
            .iter()
            .find(|(entity, owner)| owner.0 == timeline && self.targets.displays(timeline, *entity))
            .map(|(entity, _)| entity)
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
                toasts.info(t!("tab.timeline.pending.cancelled"));
            }
        }
        world.spawn((bundle, PendingOf(owner), status));
    });
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
