use crate::schedule::EditorSet;
use crate::selection::{handle_select_event, Select};
use crate::timeline::pending::PendingOf;
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
            select_clicked
                .in_set(EditorSet::Edit)
                .before(handle_select_event),
        );
    }
}

/// Clicks on a timeline belong to its pending object
fn select_clicked(
    mut clicks: MessageReader<TimelineClick>,
    mut select: MessageWriter<Select>,
    pending: Option<Single<&PendingOf>>,
) {
    for click in clicks.read() {
        if pending
            .as_ref()
            .is_some_and(|owner| owner.0 == click.timeline)
        {
            continue;
        }
        select.write(Select(vec![click.entity]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection::SelectedLine;
    use crate::timeline::pending::PendingOf;
    use crate::timeline::target::FollowSelectedLine;
    use bevy::ecs::system::RunSystemOnce;

    fn click(app: &mut App, timeline: Entity, entity: Entity) -> Vec<Vec<Entity>> {
        app.world_mut()
            .write_message(TimelineClick { timeline, entity });
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
        app.world_mut().resource_mut::<Messages<Select>>().clear();
        selections
    }

    #[test]
    fn clicks_select_unless_the_timeline_owns_a_pending_object() {
        let mut app = App::new();
        app.add_plugins(TimelineClickPlugin).add_message::<Select>();
        let world = app.world_mut();
        let line = world.spawn_empty().id();
        world.insert_resource(SelectedLine(line));
        let owner = world.spawn(FollowSelectedLine).id();
        let free = world.spawn(FollowSelectedLine).id();
        world.spawn((PendingOf(owner), ChildOf(line)));
        let entity = world.spawn_empty().id();

        assert!(click(&mut app, owner, entity).is_empty());
        assert_eq!(click(&mut app, free, entity), vec![vec![entity]]);
    }
}
