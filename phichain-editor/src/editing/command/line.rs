use crate::chart::event::LineEventBundle;
use crate::chart::line::LineBundle;
use crate::chart::note::NoteBundle;
use crate::serialization::LineWrapper;
use bevy::prelude::*;
use undo::Edit;

#[derive(Debug, Copy, Clone)]
pub struct CreateLine(Option<Entity>);

impl CreateLine {
    pub fn new() -> Self {
        Self(None)
    }
}

impl Edit for CreateLine {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        let entity = target
            .spawn(LineBundle::new())
            .with_children(|parent| {
                for event in LineWrapper::default().events {
                    parent.spawn(LineEventBundle::new(event));
                }
            })
            .id();
        self.0 = Some(entity);
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some(entity) = self.0 {
            target.entity_mut(entity).despawn_recursive();
        }
    }
}

#[derive(Debug, Clone)]
pub struct RemoveLine {
    entity: Entity,
    line: Option<LineWrapper>,
}

impl RemoveLine {
    pub fn new(entity: Entity) -> Self {
        Self { entity, line: None }
    }
}

impl Edit for RemoveLine {
    type Target = World;
    type Output = ();

    // To persist entity ID for each line, we do not despawn the line entity directly
    // Instead, we retain the entity, despawn all its children and remove all components
    // When undoing, we restore the line entity and its children
    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        self.line = Some(LineWrapper::serialize_line(target, self.entity));

        // despawn all children
        if let Some(children) = target.entity_mut(self.entity).take::<Children>() {
            for child in children.iter() {
                target.entity_mut(*child).despawn_recursive();
            }
        }

        // remove all components
        target.entity_mut(self.entity).retain::<()>();
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some(ref line) = self.line {
            // restore line entity and its children
            target
                .entity_mut(self.entity)
                .insert(LineBundle::new())
                .with_children(|parent| {
                    for note in &line.notes {
                        parent.spawn(NoteBundle::new(*note));
                    }
                    for event in &line.events {
                        parent.spawn(LineEventBundle::new(*event));
                    }
                });
        }
    }
}
