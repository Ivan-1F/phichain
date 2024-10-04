use bevy::prelude::*;
use phichain_chart::event::LineEventBundle;
use phichain_chart::line::LineBundle;
use phichain_chart::note::NoteBundle;
use phichain_chart::serialization::LineWrapper;
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
            .spawn(LineBundle::default())
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
                .insert(LineBundle::new(line.line.clone()))
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

/// Move a line as child of another line
#[derive(Debug, Clone)]
pub struct MoveLineAsChild {
    entity: Entity,
    prev_parent: Option<Entity>,
    /// Some = move as child of this line, None = move to root
    target: Option<Entity>,
}

impl MoveLineAsChild {
    pub fn new(entity: Entity, target: Option<Entity>) -> Self {
        Self {
            entity,
            prev_parent: None,
            target,
        }
    }
}

impl Edit for MoveLineAsChild {
    type Target = World;
    type Output = ();

    fn edit(&mut self, world: &mut Self::Target) -> Self::Output {
        self.prev_parent = world.entity(self.entity).get::<Parent>().map(|x| x.get());
        match self.target {
            None => {
                world.entity_mut(self.entity).remove_parent();
            }
            Some(target) => {
                world.entity_mut(self.entity).set_parent(target);
            }
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        target.entity_mut(self.entity).remove_parent();
        if let Some(prev_parent) = self.prev_parent {
            target.entity_mut(self.entity).set_parent(prev_parent);
        }
    }
}
