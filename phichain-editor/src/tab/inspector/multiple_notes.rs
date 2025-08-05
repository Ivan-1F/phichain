use crate::editing::command::note::EditNote;
use crate::editing::command::{CommandSequence, EditorCommand};
use crate::editing::DoCommandEvent;
use crate::selection::Selected;
use bevy::prelude::*;
use egui::{Align, Layout, Ui};
use phichain_chart::beat;
use phichain_chart::note::{Note, NoteKind};

pub fn multiple_notes_inspector(
    In(mut ui): In<Ui>,
    query: Query<(&Note, Entity), With<Selected>>,
    mut event_writer: EventWriter<DoCommandEvent>,
) -> Result {
    ui.label(t!(
        "tab.inspector.multiple_notes.title",
        amount = query.iter().len()
    ));
    ui.separator();

    ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
        if ui
            .button(t!("tab.inspector.multiple_notes.flip_by_x"))
            .clicked()
        {
            let commands = query
                .iter()
                .map(|(note, entity)| {
                    EditorCommand::EditNote(EditNote::new(
                        entity,
                        *note,
                        Note {
                            x: -note.x,
                            ..*note
                        },
                    ))
                })
                .collect::<Vec<_>>();

            event_writer.write(DoCommandEvent(EditorCommand::CommandSequence(
                CommandSequence(commands),
            )));
        }
        if ui
            .button(t!("tab.inspector.multiple_notes.flip_by_selection"))
            .clicked()
        {
            let x_sum: f32 = query.iter().map(|(note, _)| note.x).sum();
            let x_avg = x_sum / query.iter().len() as f32;

            let commands = query
                .iter()
                .map(|(note, entity)| {
                    EditorCommand::EditNote(EditNote::new(
                        entity,
                        *note,
                        Note {
                            x: 2.0 * x_avg - note.x,
                            ..*note
                        },
                    ))
                })
                .collect::<Vec<_>>();

            event_writer.write(DoCommandEvent(EditorCommand::CommandSequence(
                CommandSequence(commands),
            )));
        }

        if ui
            .button(t!("tab.inspector.multiple_notes.flip_side"))
            .clicked()
        {
            let commands = query
                .iter()
                .map(|(note, entity)| {
                    EditorCommand::EditNote(EditNote::new(
                        entity,
                        *note,
                        Note {
                            above: !note.above,
                            ..*note
                        },
                    ))
                })
                .collect::<Vec<_>>();

            event_writer.write(DoCommandEvent(EditorCommand::CommandSequence(
                CommandSequence(commands),
            )));
        }

        let mut into_kind = |kind: NoteKind| {
            let commands = query
                .iter()
                .map(|(note, entity)| {
                    EditorCommand::EditNote(EditNote::new(entity, *note, Note { kind, ..*note }))
                })
                .collect::<Vec<_>>();

            event_writer.write(DoCommandEvent(EditorCommand::CommandSequence(
                CommandSequence(commands),
            )));
        };

        if ui
            .button(t!("tab.inspector.multiple_notes.into_tap"))
            .clicked()
        {
            into_kind(NoteKind::Tap);
        }
        if ui
            .button(t!("tab.inspector.multiple_notes.into_drag"))
            .clicked()
        {
            into_kind(NoteKind::Drag);
        }
        if ui
            .button(t!("tab.inspector.multiple_notes.into_flick"))
            .clicked()
        {
            into_kind(NoteKind::Flick);
        }
        if ui
            .button(t!("tab.inspector.multiple_notes.into_hold"))
            .clicked()
        {
            into_kind(NoteKind::Hold {
                hold_beat: beat!(1, 32),
            });
        }
    });

    Ok(())
}
