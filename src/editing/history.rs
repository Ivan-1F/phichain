use crate::editing::command::EditorCommand;
use bevy::prelude::*;
use undo::History;

#[derive(Resource, Default)]
pub struct EditorHistory(pub History<EditorCommand>);

impl EditorHistory {
    pub fn edit(&mut self, world: &mut World, edit: EditorCommand) {
        info!("Executing command {:?}", edit);
        self.0.edit(world, edit);
    }

    pub fn undo(&mut self, world: &mut World) {
        info!("Undo");
        self.0.undo(world);
    }

    pub fn redo(&mut self, world: &mut World) {
        info!("Redo");
        self.0.redo(world);
    }
}
