use crate::editing::command::note::EditNote;
use crate::editing::command::EditorCommand;
use crate::editing::DoCommandEvent;
use crate::selection::Selected;
use crate::ui::latch;
use crate::ui::sides::SidesExt;
use crate::ui::widgets::beat_value::{BeatExt, BeatValue};
use bevy::prelude::*;
use egui::{DragValue, Ui};
use phichain_chart::note::{Note, NoteKind};

pub fn single_note_inspector(
    In(mut ui): In<Ui>,

    note: Single<(&mut Note, Entity), With<Selected>>,
    mut event_writer: EventWriter<DoCommandEvent>,
) -> Result {
    let (mut note, entity) = note.into_inner();

    ui.label(t!("tab.inspector.single_note.title", kind = note.kind));
    ui.separator();

    let result = latch::latch(&mut ui, "note", *note, |ui| {
        let mut finished = false;

        ui.sides(
            |ui| ui.label(t!("tab.inspector.single_note.beat")),
            |ui| {
                let response = ui.add(BeatValue::new(&mut note.beat).reversed(true));
                finished |= response.drag_stopped() || response.lost_focus();
            },
        );

        ui.sides(
            |ui| ui.label(t!("tab.inspector.single_note.x")),
            |ui| {
                let response = ui.add(DragValue::new(&mut note.x).speed(1));
                finished |= response.drag_stopped() || response.lost_focus();
            },
        );

        if let NoteKind::Hold { hold_beat } = note.kind {
            ui.sides(
                |ui| ui.label(t!("tab.inspector.single_note.hold_beat")),
                |ui| {
                    let mut bind = hold_beat;
                    let response = ui.add(BeatValue::new(&mut bind).reversed(true));
                    finished |= response.drag_stopped() || response.lost_focus();
                    if bind != hold_beat {
                        note.kind = NoteKind::Hold { hold_beat: bind };
                    }
                },
            );
        }

        ui.sides(
            |ui| ui.label(t!("tab.inspector.single_note.above")),
            |ui| {
                let response = ui.checkbox(&mut note.above, "");
                finished |= response.changed();
            },
        );

        ui.sides(
            |ui| ui.label(t!("tab.inspector.single_note.speed")),
            |ui| {
                let response = ui.add(DragValue::new(&mut note.speed).speed(0.1));
                finished |= response.drag_stopped() || response.lost_focus();
            },
        );

        finished
    });

    if let Some(from) = result {
        if from != *note {
            event_writer.write(DoCommandEvent(EditorCommand::EditNote(EditNote::new(
                entity, from, *note,
            ))));
        }
    }

    Ok(())
}
