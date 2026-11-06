use crate::schedule::EditorSet;
use crate::selection::{handle_select_event, Select};
use crate::timeline::pending::{CancelPendingNotOwnedBy, TimelinePending};
use bevy::prelude::*;

/// An object drawn on a timeline was clicked
#[derive(Message, Debug, Clone, Copy)]
pub struct TimelineClick {
    pub timeline: Entity,
    pub entity: Entity,
}

pub struct TimelineClickPlugin;

impl Plugin for TimelineClickPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TimelineClick>().add_systems(
            Update,
            (
                cancel_pending_on_click.in_set(EditorSet::Edit),
                select_clicked
                    .in_set(EditorSet::Edit)
                    .before(handle_select_event),
            ),
        );
    }
}

fn cancel_pending_on_click(mut clicks: MessageReader<TimelineClick>, mut commands: Commands) {
    for click in clicks.read() {
        commands.queue(CancelPendingNotOwnedBy(Some(click.timeline)));
    }
}

/// Clicks on a timeline with a pending object belong to that object
fn select_clicked(
    mut clicks: MessageReader<TimelineClick>,
    mut select: MessageWriter<Select>,
    pending: Query<&TimelinePending>,
) {
    for click in clicks.read() {
        if pending
            .get(click.timeline)
            .is_ok_and(|pending| !pending.is_empty())
        {
            continue;
        }
        select.write(Select(vec![click.entity]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::pending::PendingOf;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn clicks_select_unless_the_timeline_has_a_pending_object() {
        let mut app = App::new();
        app.add_plugins(TimelineClickPlugin).add_message::<Select>();
        let world = app.world_mut();
        let owner = world.spawn_empty().id();
        let free = world.spawn_empty().id();
        world.spawn(PendingOf(owner));
        let entity = world.spawn_empty().id();

        for timeline in [owner, free] {
            app.world_mut()
                .write_message(TimelineClick { timeline, entity });
        }
        app.update();

        let selections = app
            .world_mut()
            .run_system_once(|mut select: MessageReader<Select>| {
                select
                    .read()
                    .map(|select| select.0.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap();
        assert_eq!(selections, vec![vec![entity]]);
    }

    #[test]
    fn clicking_another_timeline_cancels_the_pending_object() {
        let mut app = App::new();
        app.add_plugins(TimelineClickPlugin).add_message::<Select>();
        let world = app.world_mut();
        let owner = world.spawn_empty().id();
        let other = world.spawn_empty().id();
        let pending = world.spawn(PendingOf(owner)).id();
        let entity = world.spawn_empty().id();

        app.world_mut().write_message(TimelineClick {
            timeline: owner,
            entity,
        });
        app.update();
        assert!(app.world().get_entity(pending).is_ok());

        app.world_mut().write_message(TimelineClick {
            timeline: other,
            entity,
        });
        app.update();
        assert!(app.world().get_entity(pending).is_err());
    }
}
