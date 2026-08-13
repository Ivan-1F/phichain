use crate::events::event::{DespawnLineEventEvent, SpawnLineEventEvent};
use crate::events::EditorEvent;
use bevy::prelude::*;
use phichain_chart::event::LineEvent;
use phichain_chart::id::EventId;
use phichain_game::event::EventOf;
use undo::Edit;

#[derive(Debug, Copy, Clone)]
pub struct CreateEvent {
    pub line_entity: Entity,
    pub event: LineEvent,
    pub event_id: EventId,

    pub event_entity: Option<Entity>,
}

impl CreateEvent {
    pub fn new(line: Entity, event: LineEvent) -> Self {
        Self {
            line_entity: line,
            event,
            event_id: EventId::new(),

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
            .id(self.event_id)
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
    pub event: Option<(LineEvent, EventId, Entity)>,
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
        let event_id = target.entity(self.entity).get::<EventId>().copied();
        let line = target
            .entity(self.entity)
            .get::<EventOf>()
            .map(|x| x.target());
        self.event = Some((event.unwrap(), event_id.unwrap(), line.unwrap()));
        DespawnLineEventEvent::builder()
            .target(self.entity)
            .keep_entity(true)
            .build()
            .run(target);
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some((event, event_id, line_entity)) = self.event {
            SpawnLineEventEvent::builder()
                .target(self.entity)
                .event(event)
                .id(event_id)
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
