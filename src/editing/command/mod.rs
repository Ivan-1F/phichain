pub mod bpm_list;
pub mod event;
pub mod line;
pub mod note;

use crate::editing::command::bpm_list::{CreateBpmPoint, EditBpmPoint, RemoveBpmPoint};
use crate::editing::command::event::{CreateEvent, EditEvent, RemoveEvent};
use crate::editing::command::line::{CreateLine, RemoveLine};
use crate::editing::command::note::{CreateNote, EditNote, RemoveNote};
use bevy::prelude::*;
use undo::Edit;

#[derive(Debug, Clone)]
pub enum EditorCommand {
    CreateNote(CreateNote),
    RemoveNote(RemoveNote),
    EditNote(EditNote),

    CreateEvent(CreateEvent),
    RemoveEvent(RemoveEvent),
    EditEvent(EditEvent),

    CreateLine(CreateLine),
    RemoveLine(RemoveLine),

    CreateBpmPoint(CreateBpmPoint),
    RemoveBpmPoint(RemoveBpmPoint),
    EditBpmPoint(EditBpmPoint),

    CommandSequence(CommandSequence),
}

#[derive(Debug, Clone)]
pub struct CommandSequence(pub Vec<EditorCommand>);

impl Edit for CommandSequence {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        for command in self.0.iter_mut() {
            command.edit(target);
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        for command in self.0.iter_mut().rev() {
            command.undo(target);
        }
    }
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

impl_edit_for_command!(
    CreateNote,
    RemoveNote,
    EditNote,
    CreateEvent,
    RemoveEvent,
    EditEvent,
    CreateLine,
    RemoveLine,
    CreateBpmPoint,
    RemoveBpmPoint,
    EditBpmPoint,
    CommandSequence
);
