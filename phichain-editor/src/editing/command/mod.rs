pub mod bpm_list;
pub mod curve_note_track;
pub mod event;
pub mod line;
pub mod meta;
pub mod note;

use crate::editing::command::bpm_list::{CreateBpmPoint, EditBpmPoint, RemoveBpmPoint};
use crate::editing::command::curve_note_track::{CreateCurveNoteTrack, RemoveCurveNoteTrack};
use crate::editing::command::event::{CreateEvent, EditEvent, RemoveEvent};
use crate::editing::command::line::{CreateLine, MoveLineAsChild, RemoveLine};
use crate::editing::command::meta::{EditMeta, EditOffset};
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
    MoveLineAsChild(MoveLineAsChild),

    CreateCurveNoteTrack(CreateCurveNoteTrack),
    RemoveCurveNoteTrack(RemoveCurveNoteTrack),

    CreateBpmPoint(CreateBpmPoint),
    RemoveBpmPoint(RemoveBpmPoint),
    EditBpmPoint(EditBpmPoint),

    EditMeta(EditMeta),
    EditOffset(EditOffset),

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

// TODO: use enum_dispatch
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
    MoveLineAsChild,
    CreateCurveNoteTrack,
    RemoveCurveNoteTrack,
    CreateBpmPoint,
    RemoveBpmPoint,
    EditBpmPoint,
    EditMeta,
    EditOffset,
    CommandSequence
);
