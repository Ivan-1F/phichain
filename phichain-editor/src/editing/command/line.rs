use crate::events::line::{DespawnLineEvent, SpawnLineEvent};
use crate::events::EditorEvent;
use crate::removed::RemovedExt;
use bevy::prelude::*;
use phichain_chart::serialization::SerializedLine;
use undo::Edit;

#[derive(Debug, Copy, Clone)]
pub struct CreateLine(Option<Entity>);

impl CreateLine {
    pub fn new() -> Self {
        Self(None)
    }
    pub fn with_target(target: Entity) -> Self {
        Self(Some(target))
    }
}

impl Edit for CreateLine {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        let entity = SpawnLineEvent::builder()
            .line(SerializedLine::default())
            .maybe_target(self.0)
            .build()
            .run(target);
        self.0 = Some(entity);
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some(entity) = self.0 {
            DespawnLineEvent::builder()
                .target(entity)
                .keep_entity(true)
                .build()
                .run(target);
        }
    }
}

#[derive(Debug, Clone)]
pub struct RemoveLine {
    entity: Entity,
}

impl RemoveLine {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

impl Edit for RemoveLine {
    type Target = World;
    type Output = ();

    // To persist entity ID for each line, we do not despawn the line entity directly
    // Instead, we retain the entity, despawn all its children and remove all components
    // When undoing, we restore the line entity and its children
    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        target
            .entity_mut(self.entity)
            .increase_removed::<Children>();
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        target
            .entity_mut(self.entity)
            .decrease_removed::<Children>();
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
        self.prev_parent = world
            .entity(self.entity)
            .get::<ChildOf>()
            .map(|x| x.parent());
        match self.target {
            None => {
                world.entity_mut(self.entity).remove::<ChildOf>();
            }
            Some(target) => {
                world.entity_mut(self.entity).insert(ChildOf(target));
            }
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        target.entity_mut(self.entity).remove::<ChildOf>();
        if let Some(prev_parent) = self.prev_parent {
            target.entity_mut(self.entity).insert(ChildOf(prev_parent));
        }
    }
}
