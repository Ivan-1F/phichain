use crate::project::{Project, ProjectMeta};
use bevy::prelude::World;
use phichain_chart::offset::Offset;
use undo::Edit;

#[derive(Debug, Clone)]
pub struct EditMeta {
    from: ProjectMeta,
    to: ProjectMeta,
}

impl EditMeta {
    pub fn new(from: ProjectMeta, to: ProjectMeta) -> Self {
        Self { from, to }
    }
}

impl Edit for EditMeta {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        let mut project = target.resource_mut::<Project>();
        project.meta = self.to.clone();
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        let mut project = target.resource_mut::<Project>();
        project.meta = self.from.clone();
    }
}

#[derive(Debug, Copy, Clone)]
pub struct EditOffset {
    from: f32,
    to: f32,
}

impl EditOffset {
    pub fn new(from: f32, to: f32) -> Self {
        Self { from, to }
    }
}

impl Edit for EditOffset {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        let mut offset = target.resource_mut::<Offset>();
        offset.0 = self.to;
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        let mut offset = target.resource_mut::<Offset>();
        offset.0 = self.from;
    }
}
