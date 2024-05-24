use crate::chart::event::LineEventBundle;
use crate::chart::line::LineBundle;
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
                for event in LineWrapper::default().1 {
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
