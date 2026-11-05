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

/// Minimum share of the whole timeline viewport a timeline can be resized to
const MIN_TIMELINE_SHARE: f32 = 0.05;

/// A timeline on the timeline tab
#[derive(Component, Debug)]
pub struct Timeline {
    /// Relative width, the viewport is split in proportion to the widths of all timelines
    pub width: f32,
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
        let widths: Vec<_> = contained
            .iter()
            .filter_map(|timeline| Some((timeline, self.timelines.get(timeline).ok()?.width)))
            .collect();
        let total: f32 = widths.iter().map(|(_, width)| width).sum();
        let mut left = viewport.left();
        widths
            .into_iter()
            .map(|(timeline, width)| {
                let mut allocated = viewport;
                allocated.set_left(left);
                allocated.set_right(left + width / total * viewport.width());
                left = allocated.right();
                AllocatedTimeline {
                    timeline,
                    target: self.targets.get(timeline),
                    viewport: allocated,
                }
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
        Timeline { width: 2.0 },
        InContainer(container),
    ));
    world.spawn((
        EventTimeline,
        FollowSelectedLine,
        Timeline { width: 1.0 },
        InContainer(container),
    ));
}

/// Push a new timeline showing the given target to the right of the container
///
/// The new timeline takes the average width of the existing ones.
/// A `None` target makes the timeline follow the selected line.
pub fn push_timeline(world: &mut World, kind: impl Bundle, target: Option<Entity>) {
    let Some(container) = container(world) else {
        return;
    };
    let widths: Vec<f32> = world
        .query::<&Timeline>()
        .iter(world)
        .map(|timeline| timeline.width)
        .collect();
    let width = if widths.is_empty() {
        1.0
    } else {
        widths.iter().sum::<f32>() / widths.len() as f32
    };
    let mut timeline = world.spawn((kind, Timeline { width }, InContainer(container)));
    match target {
        Some(target) => timeline.insert(TimelineOf(target)),
        None => timeline.insert(FollowSelectedLine),
    };
}

/// Swap two timelines, each keeping its width
pub fn swap_timelines(world: &mut World, a: usize, b: usize) {
    let Some(mut contained) =
        container(world).and_then(|container| world.get_mut::<ContainedTimelines>(container))
    else {
        return;
    };
    if a < contained.len() && b < contained.len() {
        contained.swap(a, b);
    }
}

/// Move the separator between the timeline at `index` and the next one
///
/// `by` is a fraction of the whole timeline viewport
pub fn move_separator(world: &mut World, index: usize, by: f32) {
    let timelines = ordered_timelines(world);
    let (Some(&left), Some(&right)) = (timelines.get(index), timelines.get(index + 1)) else {
        return;
    };
    let width = |entity: Entity| world.get::<Timeline>(entity).unwrap().width;
    let total: f32 = timelines.iter().map(|entity| width(*entity)).sum();
    let min = MIN_TIMELINE_SHARE * total;
    let delta = (by * total).max(min - width(left)).min(width(right) - min);
    world.get_mut::<Timeline>(left).unwrap().width += delta;
    world.get_mut::<Timeline>(right).unwrap().width -= delta;
}

#[cfg(test)]
mod tests {
    use super::*;
    use phichain_chart::line::Line;

    fn world_with_container() -> World {
        let mut world = World::new();
        spawn_timeline_container(&mut world);
        world
    }

    fn widths(world: &mut World) -> Vec<f32> {
        ordered_timelines(world)
            .into_iter()
            .map(|timeline| world.get::<Timeline>(timeline).unwrap().width)
            .collect()
    }

    fn edges(world: &mut World) -> Vec<(f32, f32)> {
        let viewport = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(300.0, 100.0));
        allocate_timelines(world, viewport)
            .into_iter()
            .map(|item| (item.viewport.left(), item.viewport.right()))
            .collect()
    }

    fn assert_approx(actual: Vec<f32>, expected: Vec<f32>) {
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(&expected) {
            assert!(
                (actual - expected).abs() < 1e-4,
                "{actual:?} != {expected:?}"
            );
        }
    }

    #[test]
    fn allocates_in_proportion_to_widths() {
        let mut world = world_with_container();
        assert_eq!(edges(&mut world), vec![(0.0, 200.0), (200.0, 300.0)]);
    }

    #[test]
    fn despawning_the_target_line_removes_its_timeline() {
        let mut world = world_with_container();
        let line = world.spawn(Line::default()).id();
        push_timeline(&mut world, NoteTimeline, Some(line));
        let fixed = *ordered_timelines(&mut world).last().unwrap();
        assert_eq!(world.get::<TimelineOf>(fixed).unwrap().0, line);

        world.despawn(line);

        assert!(world.get_entity(fixed).is_err());
        assert_eq!(edges(&mut world), vec![(0.0, 200.0), (200.0, 300.0)]);
    }

    #[test]
    fn a_pushed_timeline_takes_the_average_width() {
        let mut world = world_with_container();
        push_timeline(&mut world, EventTimeline, None);
        assert_eq!(widths(&mut world), vec![2.0, 1.0, 1.5]);
    }

    #[test]
    fn swapped_timelines_keep_their_widths() {
        let mut world = world_with_container();
        let before = ordered_timelines(&mut world);

        swap_timelines(&mut world, 0, 1);

        assert_eq!(ordered_timelines(&mut world), vec![before[1], before[0]]);
        assert_eq!(edges(&mut world), vec![(0.0, 100.0), (100.0, 300.0)]);
    }

    #[test]
    fn moving_a_separator_only_resizes_its_neighbors() {
        let mut world = world_with_container();
        push_timeline(&mut world, EventTimeline, None);

        move_separator(&mut world, 0, 0.1);
        assert_approx(widths(&mut world), vec![2.45, 0.55, 1.5]);

        move_separator(&mut world, 0, -1.0);
        assert_approx(widths(&mut world), vec![0.225, 2.775, 1.5]);

        move_separator(&mut world, 1, 1.0);
        assert_approx(widths(&mut world), vec![0.225, 4.05, 0.225]);
    }
}
