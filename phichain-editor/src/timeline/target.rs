use crate::selection::SelectedLine;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use phichain_game::event::EventOf;

/// The entity a timeline shows, fixed regardless of the selected line
#[derive(Component, Debug)]
#[relationship(relationship_target = Timelines)]
pub struct TimelineOf(pub Entity);

#[derive(Component, Deref, Debug)]
#[relationship_target(relationship = TimelineOf, linked_spawn)]
pub struct Timelines(Vec<Entity>);

/// The timeline shows whichever line is selected
///
/// Mutually exclusive with [`TimelineOf`]
#[derive(Component, Debug)]
pub struct FollowSelectedLine;

#[derive(SystemParam)]
pub struct TimelineTargets<'w, 's> {
    selected_line: Option<Res<'w, SelectedLine>>,
    fixed: Query<'w, 's, &'static TimelineOf>,
    follow: Query<'w, 's, (), With<FollowSelectedLine>>,
    placements: Query<'w, 's, (Option<&'static ChildOf>, Option<&'static EventOf>)>,
}

impl TimelineTargets<'_, '_> {
    /// The entity the timeline currently shows
    pub fn get(&self, timeline: Entity) -> Option<Entity> {
        if let Ok(of) = self.fixed.get(timeline) {
            return Some(of.0);
        }
        if self.follow.contains(timeline) {
            return self.selected_line.as_ref().map(|line| line.0);
        }
        None
    }

    /// The entity a note, event or curve note track is placed on
    pub fn placement(&self, entity: Entity) -> Option<Entity> {
        let (child_of, event_of) = self.placements.get(entity).ok()?;
        child_of
            .map(ChildOf::parent)
            .or(event_of.map(|event_of| event_of.0))
    }

    /// Whether the timeline shows the entity, which is placed on its current target
    pub fn displays(&self, timeline: Entity, entity: Entity) -> bool {
        self.placement(entity)
            .is_some_and(|placement| self.get(timeline) == Some(placement))
    }
}
