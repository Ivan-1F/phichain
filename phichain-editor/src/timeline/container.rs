use crate::timeline::event::EventTimeline;
use crate::timeline::note::NoteTimeline;
use crate::timeline::target::{FollowSelectedLine, TimelineOf, TimelineTargets};
use bevy::ecs::system::{SystemParam, SystemState};
use bevy::prelude::*;
use egui::Rect;

/// Holds all the timelines on the timeline tab
#[derive(Component, Debug)]
pub struct TimelineContainer;

#[derive(Component, Debug)]
#[relationship(relationship_target = ContainedTimelines)]
pub struct InContainer(pub Entity);

/// Timelines of a [`TimelineContainer`] from left to right
#[derive(Component, Deref, Debug)]
#[relationship_target(relationship = InContainer)]
pub struct ContainedTimelines(Vec<Entity>);

impl ContainedTimelines {
    pub fn swap(&mut self, a: usize, b: usize) {
        self.0.swap(a, b);
    }
}

/// A timeline on the timeline tab
#[derive(Component, Debug)]
pub struct Timeline {
    /// Position of the right edge, as a fraction of the whole timeline viewport
    ///
    /// Fractions are ascending from left to right, the last one is always 1.0
    pub fraction: f32,
}

/// A timeline with the entity it shows and its allocated viewport
#[derive(Debug, Clone, Copy)]
pub struct AllocatedTimeline {
    pub timeline: Entity,
    pub target: Option<Entity>,
    pub viewport: Rect,
}

#[derive(SystemParam)]
pub struct TimelineLayout<'w, 's> {
    container: Query<'w, 's, &'static ContainedTimelines, With<TimelineContainer>>,
    timelines: Query<'w, 's, &'static Timeline>,
    targets: TimelineTargets<'w, 's>,
}

impl TimelineLayout<'_, '_> {
    /// Allocate all the timelines in the given viewport
    pub fn allocate(&self, viewport: Rect) -> Vec<AllocatedTimeline> {
        let Ok(contained) = self.container.single() else {
            return vec![];
        };
        let mut left = viewport.left();
        contained
            .iter()
            .filter_map(|timeline| {
                let fraction = self.timelines.get(timeline).ok()?.fraction;
                let mut allocated = viewport;
                allocated.set_left(left);
                allocated.set_right(viewport.left() + fraction * viewport.width());
                left = allocated.right();
                Some(AllocatedTimeline {
                    timeline,
                    target: self.targets.get(timeline),
                    viewport: allocated,
                })
            })
            .collect()
    }
}

/// Allocate all the timelines in the given viewport
pub fn allocate_timelines(world: &mut World, viewport: Rect) -> Vec<AllocatedTimeline> {
    let mut state: SystemState<TimelineLayout> = SystemState::new(world);
    state.get(world).allocate(viewport)
}

fn container(world: &mut World) -> Option<Entity> {
    world
        .query_filtered::<Entity, With<TimelineContainer>>()
        .single(world)
        .ok()
}

/// All the timelines from left to right
pub fn ordered_timelines(world: &mut World) -> Vec<Entity> {
    container(world)
        .and_then(|container| world.get::<ContainedTimelines>(container))
        .map(|contained| contained.to_vec())
        .unwrap_or_default()
}

pub fn spawn_timeline_container(world: &mut World) {
    let container = world.spawn(TimelineContainer).id();
    world.spawn((
        NoteTimeline,
        FollowSelectedLine,
        Timeline {
            fraction: 2.0 / 3.0,
        },
        InContainer(container),
    ));
    world.spawn((
        EventTimeline,
        FollowSelectedLine,
        Timeline { fraction: 1.0 },
        InContainer(container),
    ));
}

/// Push a new timeline showing the given target to the right of the container
///
/// A `None` target makes the timeline follow the selected line
pub fn push_timeline(world: &mut World, kind: impl Bundle, target: Option<Entity>) {
    let Some(container) = container(world) else {
        return;
    };
    for mut timeline in world.query::<&mut Timeline>().iter_mut(world) {
        timeline.fraction /= 1.2;
    }
    let mut timeline = world.spawn((kind, Timeline { fraction: 1.0 }, InContainer(container)));
    match target {
        Some(target) => timeline.insert(TimelineOf(target)),
        None => timeline.insert(FollowSelectedLine),
    };
}

/// Swap two timelines as well as their fractions
pub fn swap_timelines(world: &mut World, a: usize, b: usize) {
    let timelines = ordered_timelines(world);
    let (Some(&a_entity), Some(&b_entity)) = (timelines.get(a), timelines.get(b)) else {
        return;
    };
    let a_fraction = world.get::<Timeline>(a_entity).unwrap().fraction;
    let b_fraction = world.get::<Timeline>(b_entity).unwrap().fraction;
    world.get_mut::<Timeline>(a_entity).unwrap().fraction = b_fraction;
    world.get_mut::<Timeline>(b_entity).unwrap().fraction = a_fraction;
    if let Some(container) = container(world) {
        world
            .get_mut::<ContainedTimelines>(container)
            .unwrap()
            .swap(a, b);
    }
}

/// Offset the fraction of a timeline at the given index by some delta
pub fn offset_timeline(world: &mut World, index: usize, by: f32) {
    let timelines = ordered_timelines(world);
    let fraction_at = |index: Option<usize>| {
        index
            .and_then(|index| timelines.get(index))
            .and_then(|entity| world.get::<Timeline>(*entity))
            .map(|timeline| timeline.fraction)
    };
    let last = fraction_at(index.checked_sub(1));
    let next = fraction_at(Some(index + 1));
    let Some(mut timeline) = timelines
        .get(index)
        .and_then(|entity| world.get_mut::<Timeline>(*entity))
    else {
        return;
    };

    let new_fraction = timeline.fraction + by;
    timeline.fraction = new_fraction
        .max(last.unwrap_or(0.0) + 0.05)
        .min(next.map(|x| x - 0.05).unwrap_or(1.0));
}

/// Let the last remaining timeline fill the space left by a removed timeline
pub(super) fn fill_removed_timeline_observer(
    remove: On<Remove, Timeline>,
    containers: Query<&ContainedTimelines>,
    mut timelines: Query<&mut Timeline>,
) {
    for contained in &containers {
        let last = contained
            .iter()
            .rev()
            .find(|entity| *entity != remove.entity);
        if let Some(mut timeline) = last.and_then(|entity| timelines.get_mut(entity).ok()) {
            timeline.fraction = 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phichain_chart::line::Line;

    fn world_with_container() -> World {
        let mut world = World::new();
        world.add_observer(fill_removed_timeline_observer);
        spawn_timeline_container(&mut world);
        world
    }

    fn fractions(world: &mut World) -> Vec<f32> {
        ordered_timelines(world)
            .into_iter()
            .map(|timeline| world.get::<Timeline>(timeline).unwrap().fraction)
            .collect()
    }

    #[test]
    fn despawning_the_target_line_removes_its_timeline_and_fills_the_space() {
        let mut world = world_with_container();
        let line = world.spawn(Line::default()).id();
        push_timeline(&mut world, NoteTimeline, Some(line));
        let fixed = *ordered_timelines(&mut world).last().unwrap();
        assert_eq!(world.get::<TimelineOf>(fixed).unwrap().0, line);

        world.despawn(line);

        assert!(world.get_entity(fixed).is_err());
        assert_eq!(fractions(&mut world), vec![2.0 / 3.0 / 1.2, 1.0]);
    }

    #[test]
    fn swapping_timelines_keeps_fractions_in_place() {
        let mut world = world_with_container();
        let before = ordered_timelines(&mut world);

        swap_timelines(&mut world, 0, 1);

        let after = ordered_timelines(&mut world);
        assert_eq!(after, vec![before[1], before[0]]);
        assert!(world.get::<EventTimeline>(after[0]).is_some());
        assert_eq!(fractions(&mut world), vec![2.0 / 3.0, 1.0]);
    }
}
