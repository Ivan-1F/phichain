use crate::chart::event::{LineEvent, LineEventBundle};
use bevy::prelude::{Entity, World};
use undo::Edit;

#[derive(Debug, Copy, Clone)]
pub struct CreateEvent(pub LineEvent, pub Option<Entity>);

impl CreateEvent {
    #[allow(dead_code)]
    pub fn new(event: LineEvent) -> Self {
        Self(event, None)
    }
}

impl Edit for CreateEvent {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        self.1 = Some(target.spawn(LineEventBundle::new(self.0)).id());
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some(entity) = self.1 {
            target.despawn(entity);
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RemoveEvent(pub Entity, pub Option<LineEvent>);

impl RemoveEvent {
    #[allow(dead_code)]
    pub fn new(entity: Entity) -> Self {
        Self(entity, None)
    }
}

impl Edit for RemoveEvent {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        self.1 = target.entity(self.0).get::<LineEvent>().copied();
        target.despawn(self.0);
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some(event) = self.1 {
            self.0 = target.spawn(LineEventBundle::new(event)).id();
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
