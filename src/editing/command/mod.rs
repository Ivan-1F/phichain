pub mod note;

use crate::editing::command::note::{CreateNote, EditNote, RemoveNote};
use bevy::prelude::*;
use undo::Edit;

pub enum EditorCommand {
    CreateNote(CreateNote),
    RemoveNote(RemoveNote),
    EditNote(EditNote),
}

macro_rules! impl_edit_for_command {
    ($($variant:ident),*) => {
        impl Edit for EditorCommand {
            type Target = World;
            type Output = ();

            fn edit(&mut self, target: &mut Self::Target) {
                match self {
                    $(
                        EditorCommand::$variant(cmd) => cmd.edit(target),
                    )*
                }
            }

            fn undo(&mut self, target: &mut Self::Target) {
                match self {
                    $(
                        EditorCommand::$variant(cmd) => cmd.undo(target),
                    )*
                }
            }
        }
    };
}

impl_edit_for_command!(CreateNote, RemoveNote, EditNote);
