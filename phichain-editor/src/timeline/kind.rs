use crate::timeline::target::TimelineOf;
use bevy::ecs::component::ComponentId;
use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use egui::{Rect, Ui};
use phichain_chart::line::Line;

/// Systems implementing a kind of timeline, identified by its marker component
#[derive(Debug, Clone, Copy)]
pub struct TimelineKind {
    marker: ComponentId,
    ui: SystemId<In<(Ui, Entity)>>,
    drag_selection: SystemId<In<(Entity, Rect)>, Vec<Entity>>,
    name: SystemId<In<Entity>, String>,
}

#[derive(Resource, Debug, Default)]
pub struct TimelineKinds(Vec<TimelineKind>);

pub trait TimelineKindRegistrationExt {
    /// Register the systems of timelines carrying the `marker` component
    fn add_timeline_kind<T, M1, M2, M3>(
        &mut self,
        marker: T,
        ui: impl IntoSystem<In<(Ui, Entity)>, (), M1> + 'static,
        drag_selection: impl IntoSystem<In<(Entity, Rect)>, Vec<Entity>, M2> + 'static,
        name: impl IntoSystem<In<Entity>, String, M3> + 'static,
    ) -> &mut Self
    where
        T: Component;
}

impl TimelineKindRegistrationExt for App {
    fn add_timeline_kind<T, M1, M2, M3>(
        &mut self,
        _marker: T,
        ui: impl IntoSystem<In<(Ui, Entity)>, (), M1> + 'static,
        drag_selection: impl IntoSystem<In<(Entity, Rect)>, Vec<Entity>, M2> + 'static,
        name: impl IntoSystem<In<Entity>, String, M3> + 'static,
    ) -> &mut Self
    where
        T: Component,
    {
        let world = self.world_mut();
        let kind = TimelineKind {
            marker: world.register_component::<T>(),
            ui: world.register_system(ui),
            drag_selection: world.register_system(drag_selection),
            name: world.register_system(name),
        };
        world.get_resource_or_init::<TimelineKinds>().0.push(kind);
        self
    }
}

fn timeline_kind(world: &World, timeline: Entity) -> Option<TimelineKind> {
    let timeline = world.get_entity(timeline).ok()?;
    world
        .get_resource::<TimelineKinds>()?
        .0
        .iter()
        .find(|kind| timeline.contains_id(kind.marker))
        .copied()
}

/// Draw the timeline into the given [`Ui`], whose max rect is the timeline viewport
pub fn timeline_ui(world: &mut World, ui: Ui, timeline: Entity) {
    let Some(kind) = timeline_kind(world, timeline) else {
        return;
    };
    if let Err(error) = world.run_system_with(kind.ui, (ui, timeline)) {
        error!("Failed to draw timeline: {error}");
    }
}

/// The entities inside the selection, a rect relative to the timeline where x is the x position and y is the time
pub fn timeline_selection(world: &mut World, timeline: Entity, selection: Rect) -> Vec<Entity> {
    let Some(kind) = timeline_kind(world, timeline) else {
        return vec![];
    };
    world
        .run_system_with(kind.drag_selection, (timeline, selection))
        .unwrap_or_else(|error| {
            error!("Failed to run drag selection on timeline: {error}");
            vec![]
        })
}

pub fn timeline_name(world: &mut World, timeline: Entity) -> String {
    let Some(kind) = timeline_kind(world, timeline) else {
        return String::new();
    };
    world
        .run_system_with(kind.name, timeline)
        .unwrap_or_else(|error| {
            error!("Failed to get timeline name: {error}");
            String::new()
        })
}

/// Name of a timeline prefixed by its icon: the name of its fixed line, or a binding label when following the selected line
pub fn target_name(
    icon: &str,
    timeline: Entity,
    timeline_of: &Query<&TimelineOf>,
    lines: &Query<&Line>,
) -> String {
    let name = match timeline_of.get(timeline) {
        Ok(of) => lines
            .get(of.0)
            .map(|line| line.name.clone())
            .unwrap_or_default(),
        Err(_) => t!("tab.timeline_setting.timelines.binding").into_owned(),
    };
    format!("{icon} {name}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Component)]
    struct First;

    #[derive(Component)]
    struct Second;

    fn kind_app() -> App {
        let mut app = App::new();
        app.add_timeline_kind(
            First,
            |_: In<(Ui, Entity)>| {},
            |In((timeline, _)): In<(Entity, Rect)>| vec![timeline],
            |_: In<Entity>| "first".to_owned(),
        )
        .add_timeline_kind(
            Second,
            |_: In<(Ui, Entity)>| {},
            |_: In<(Entity, Rect)>| vec![],
            |_: In<Entity>| "second".to_owned(),
        );
        app
    }

    #[test]
    fn dispatches_by_marker_component() {
        let mut app = kind_app();
        let world = app.world_mut();
        let first = world.spawn(First).id();
        let second = world.spawn(Second).id();

        assert_eq!(timeline_name(world, first), "first");
        assert_eq!(timeline_name(world, second), "second");
        assert_eq!(timeline_selection(world, first, Rect::ZERO), vec![first]);
        assert_eq!(timeline_selection(world, second, Rect::ZERO), vec![]);
    }

    #[test]
    fn entities_of_no_kind_get_empty_results() {
        let mut app = kind_app();
        let world = app.world_mut();
        let unknown = world.spawn_empty().id();

        assert_eq!(timeline_name(world, unknown), "");
        assert_eq!(timeline_selection(world, unknown, Rect::ZERO), vec![]);
    }
}
