use crate::tab::timeline::TimelineViewport;
use crate::timeline::event::EventTimeline;
use crate::timeline::note::NoteTimeline;
use crate::timeline::target::{FollowSelectedLine, TimelineOf, TimelineTargets};
use crate::utils::convert::BevyEguiConvert;
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

/// Minimum share of the whole timeline viewport a timeline can be resized to
const MIN_TIMELINE_SHARE: f32 = 0.05;

/// A timeline on the timeline tab
#[derive(Component, Debug)]
#[require(TimelineRect)]
pub struct Timeline {
    /// The viewport is split in proportion to the weights of all timelines
    pub weight: f32,
}

/// Viewport of a timeline from the last [`layout_timelines`]
#[derive(Component, Debug, Clone, Copy)]
pub struct TimelineRect(pub Rect);

impl Default for TimelineRect {
    fn default() -> Self {
        Self(Rect::NOTHING)
    }
}

/// A timeline with the entity it shows and its viewport from the last layout
#[derive(Debug, Clone, Copy)]
pub struct LaidOutTimeline {
    pub timeline: Entity,
    pub target: Option<Entity>,
    pub viewport: Rect,
}

#[derive(SystemParam)]
pub struct TimelineLayout<'w, 's> {
    container: Query<'w, 's, &'static ContainedTimelines, With<TimelineContainer>>,
    rects: Query<'w, 's, &'static TimelineRect>,
    targets: TimelineTargets<'w, 's>,
}

impl TimelineLayout<'_, '_> {
    /// All the timelines from left to right
    pub fn timelines(&self) -> impl Iterator<Item = LaidOutTimeline> + '_ {
        self.container
            .single()
            .into_iter()
            .flat_map(|contained| contained.iter())
            .filter_map(|timeline| {
                Some(LaidOutTimeline {
                    timeline,
                    target: self.targets.get(timeline),
                    viewport: self.rects.get(timeline).ok()?.0,
                })
            })
    }

    /// The timeline under the given position
    pub fn at(&self, pos: egui::Pos2) -> Option<LaidOutTimeline> {
        self.timelines().find(|item| item.viewport.contains(pos))
    }
}

/// All the timelines from left to right
pub fn laid_out_timelines(world: &mut World) -> Vec<LaidOutTimeline> {
    let mut state: SystemState<TimelineLayout> = SystemState::new(world);
    state.get(world).timelines().collect()
}

/// Split the timeline viewport between all the timelines in proportion to their weights
pub fn layout_timelines(world: &mut World) {
    let viewport = world.resource::<TimelineViewport>().0.into_egui();
    let timelines = ordered_timelines(world);
    let total: f32 = timelines
        .iter()
        .filter_map(|timeline| world.get::<Timeline>(*timeline))
        .map(|timeline| timeline.weight)
        .sum();
    let mut left = viewport.left();
    for timeline in timelines {
        let Some(weight) = world
            .get::<Timeline>(timeline)
            .map(|timeline| timeline.weight)
        else {
            continue;
        };
        let mut rect = viewport;
        rect.set_left(left);
        rect.set_right(left + weight / total * viewport.width());
        left = rect.right();
        world.entity_mut(timeline).insert(TimelineRect(rect));
    }
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
        Timeline { weight: 2.0 },
        InContainer(container),
    ));
    world.spawn((
        EventTimeline,
        FollowSelectedLine,
        Timeline { weight: 1.0 },
        InContainer(container),
    ));
}

/// Push a new timeline showing the given target to the right of the container
///
/// The new timeline takes the average weight of the existing ones.
/// A `None` target makes the timeline follow the selected line.
pub fn push_timeline(world: &mut World, kind: impl Bundle, target: Option<Entity>) {
    let Some(container) = container(world) else {
        return;
    };
    let weights: Vec<f32> = world
        .query::<&Timeline>()
        .iter(world)
        .map(|timeline| timeline.weight)
        .collect();
    let weight = if weights.is_empty() {
        1.0
    } else {
        weights.iter().sum::<f32>() / weights.len() as f32
    };
    let mut timeline = world.spawn((kind, Timeline { weight }, InContainer(container)));
    match target {
        Some(target) => timeline.insert(TimelineOf(target)),
        None => timeline.insert(FollowSelectedLine),
    };
}

/// Swap two timelines, each keeping its weight
pub fn swap_timelines(world: &mut World, a: Entity, b: Entity) {
    let Some(mut contained) =
        container(world).and_then(|container| world.get_mut::<ContainedTimelines>(container))
    else {
        return;
    };
    let position = |timeline| contained.iter().position(|entity| entity == timeline);
    if let (Some(a), Some(b)) = (position(a), position(b)) {
        contained.0.swap(a, b);
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
    let weight = |entity: Entity| world.get::<Timeline>(entity).unwrap().weight;
    let total: f32 = timelines.iter().map(|entity| weight(*entity)).sum();
    let min = MIN_TIMELINE_SHARE * total;
    let delta = (by * total)
        .max(min - weight(left))
        .min(weight(right) - min);
    world.get_mut::<Timeline>(left).unwrap().weight += delta;
    world.get_mut::<Timeline>(right).unwrap().weight -= delta;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world_with_container() -> World {
        let mut world = World::new();
        world.insert_resource(TimelineViewport(bevy::math::Rect::new(
            0.0, 0.0, 300.0, 100.0,
        )));
        spawn_timeline_container(&mut world);
        world
    }

    fn weights(world: &mut World) -> Vec<f32> {
        ordered_timelines(world)
            .into_iter()
            .map(|timeline| world.get::<Timeline>(timeline).unwrap().weight)
            .collect()
    }

    fn edges(world: &mut World) -> Vec<(f32, f32)> {
        layout_timelines(world);
        laid_out_timelines(world)
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
    fn splits_the_viewport_in_proportion_to_weights() {
        let mut world = world_with_container();
        assert_eq!(edges(&mut world), vec![(0.0, 200.0), (200.0, 300.0)]);
    }

    #[test]
    fn finds_the_timeline_under_a_position() {
        let mut world = world_with_container();
        layout_timelines(&mut world);
        let timelines = ordered_timelines(&mut world);
        let mut state: SystemState<TimelineLayout> = SystemState::new(&mut world);
        let layout = state.get(&world);

        let at = |x: f32| layout.at(egui::pos2(x, 50.0)).map(|item| item.timeline);
        assert_eq!(at(50.0), Some(timelines[0]));
        assert_eq!(at(250.0), Some(timelines[1]));
        assert_eq!(at(400.0), None);
    }

    #[test]
    fn a_pushed_timeline_takes_the_average_weight() {
        let mut world = world_with_container();
        push_timeline(&mut world, EventTimeline, None);
        assert_eq!(weights(&mut world), vec![2.0, 1.0, 1.5]);
    }

    #[test]
    fn swapped_timelines_keep_their_weights() {
        let mut world = world_with_container();
        let before = ordered_timelines(&mut world);

        swap_timelines(&mut world, before[0], before[1]);

        assert_eq!(ordered_timelines(&mut world), vec![before[1], before[0]]);
        assert_eq!(edges(&mut world), vec![(0.0, 100.0), (100.0, 300.0)]);
    }

    #[test]
    fn moving_a_separator_only_resizes_its_neighbors() {
        let mut world = world_with_container();
        push_timeline(&mut world, EventTimeline, None);

        move_separator(&mut world, 0, 0.1);
        assert_approx(weights(&mut world), vec![2.45, 0.55, 1.5]);

        move_separator(&mut world, 0, -1.0);
        assert_approx(weights(&mut world), vec![0.225, 2.775, 1.5]);

        move_separator(&mut world, 1, 1.0);
        assert_approx(weights(&mut world), vec![0.225, 4.05, 0.225]);
    }
}
