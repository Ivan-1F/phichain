use crate::events::event::{DespawnLineEventEvent, SpawnLineEventEvent};
use crate::events::EditorEvent;
use bevy::prelude::*;
use phichain_chart::event::LineEvent;
use undo::Edit;

#[derive(Debug, Copy, Clone)]
pub struct CreateEvent {
    pub line_entity: Entity,
    pub event: LineEvent,
    pub event_entity: Option<Entity>,
}

impl CreateEvent {
    pub fn new(line: Entity, event: LineEvent) -> Self {
        Self {
            line_entity: line,
            event,
            event_entity: None,
        }
    }
}

impl Edit for CreateEvent {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        let entity = SpawnLineEventEvent::builder()
            .event(self.event)
            .line_entity(self.line_entity)
            .maybe_target(self.event_entity)
            .build()
            .run(target);
        self.event_entity = Some(entity)
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some(entity) = self.event_entity {
            DespawnLineEventEvent::builder()
                .target(entity)
                .keep_entity(true)
                .build()
                .run(target);
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RemoveEvent {
    pub entity: Entity,
    pub event: Option<(LineEvent, Entity)>,
}

impl RemoveEvent {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            event: None,
        }
    }
}

impl Edit for RemoveEvent {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        let event = target.entity(self.entity).get::<LineEvent>().copied();
        let parent = target
            .entity(self.entity)
            .get::<ChildOf>()
            .map(|x| x.parent());
        self.event = Some((event.unwrap(), parent.unwrap()));
        DespawnLineEventEvent::builder()
            .target(self.entity)
            .keep_entity(true)
            .build()
            .run(target);
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some((event, line_entity)) = self.event {
            SpawnLineEventEvent::builder()
                .target(self.entity)
                .event(event)
                .line_entity(line_entity)
                .build()
                .run(target);
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct EditEvent {
    entity: Entity,
    from: LineEvent,
    to: LineEvent,
}

impl EditEvent {
    pub fn new(entity: Entity, from: LineEvent, to: LineEvent) -> Self {
        Self { entity, from, to }
    }
}

impl Edit for EditEvent {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some(mut event) = target.entity_mut(self.entity).get_mut::<LineEvent>() {
            *event = self.to;
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some(mut event) = target.entity_mut(self.entity).get_mut::<LineEvent>() {
            *event = self.from;
        }
    }
}

#[cfg(test)]
mod tests {}
