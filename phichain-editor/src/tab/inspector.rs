use bevy::prelude::*;
use egui::{Align, Layout, Ui};
use phichain_chart::beat;

use crate::editing::command::event::EditEvent;
use crate::editing::command::note::EditNote;
use crate::editing::command::{CommandSequence, EditorCommand};
use crate::editing::DoCommandEvent;
use crate::selection::Selected;
use crate::ui::latch;
use crate::ui::widgets::beat_value::BeatExt;
use crate::ui::widgets::easing_value::EasingValue;
use phichain_chart::event::{LineEvent, LineEventKind};
use phichain_chart::note::{Note, NoteKind};

pub fn inspector_ui_system(
    In(ui): In<&mut Ui>,
    mut selected_notes: Query<(&mut Note, Entity), With<Selected>>,
    mut selected_events: Query<(&mut LineEvent, Entity), With<Selected>>,
    event_writer: EventWriter<DoCommandEvent>,
) {
    let mut selected_notes: Vec<_> = selected_notes.iter_mut().collect();
    let mut selected_events: Vec<_> = selected_events.iter_mut().collect();
    if selected_notes.len() == 1 && selected_events.is_empty() {
        let (selected_note, entity) = selected_notes.get_mut(0).unwrap();
        single_note_inspector(ui, *entity, selected_note, event_writer);
    } else if selected_notes.is_empty() && selected_events.len() == 1 {
        let (selected_event, entity) = selected_events.get_mut(0).unwrap();
        single_event_inspector(ui, *entity, selected_event, event_writer);
    } else if selected_notes.len() > 1 && selected_events.is_empty() {
        multiple_notes_inspector(ui, &selected_notes, event_writer);
    } else if selected_notes.is_empty() && selected_events.len() > 1 {
        multiple_events_inspector(ui, &selected_events, event_writer);
    }
}

fn single_event_inspector(
    ui: &mut Ui,
    entity: Entity,
    event: &mut LineEvent,
    mut event_writer: EventWriter<DoCommandEvent>,
) {
    egui::Grid::new("inspector_grid")
        .num_columns(2)
        .spacing([20.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            let result = latch::latch(ui, "event", *event, |ui| {
                let mut finished = false;

                ui.label(t!("tab.inspector.single_event.start_beat"));
                let response = ui.beat(&mut event.start_beat);
                finished |= response.drag_stopped() || response.lost_focus();
                ui.end_row();

                ui.label(t!("tab.inspector.single_event.end_beat"));
                let response = ui.beat(&mut event.end_beat);
                finished |= response.drag_stopped() || response.lost_focus();
                ui.end_row();

                ui.label(t!("tab.inspector.single_event.start_value"));
                let range = match event.kind {
                    LineEventKind::Opacity => 0.0..=255.0,
                    _ => f32::MIN..=f32::MAX,
                };
                let response = ui.add(
                    egui::DragValue::new(&mut event.start)
                        .clamp_range(range.clone())
                        .speed(1.0),
                );
                finished |= response.drag_stopped() || response.lost_focus();
                ui.end_row();

                ui.label(t!("tab.inspector.single_event.end_value"));
                let response = ui.add(
                    egui::DragValue::new(&mut event.end)
                        .clamp_range(range.clone())
                        .speed(1.0),
                );
                finished |= response.drag_stopped() || response.lost_focus();
                ui.end_row();

                ui.label(t!("tab.inspector.single_event.end_value"));
                let response = ui.add(EasingValue::new(&mut event.easing));
                finished |= response.drag_stopped() || response.lost_focus();
                ui.end_row();

                finished
            });

            if let Some(from) = result {
                if from != *event {
                    event_writer.send(DoCommandEvent(EditorCommand::EditEvent(EditEvent::new(
                        entity, from, *event,
                    ))));
                }
            }
        });
}

fn single_note_inspector(
    ui: &mut Ui,
    entity: Entity,
    note: &mut Note,
    mut event_writer: EventWriter<DoCommandEvent>,
) {
    egui::Grid::new("inspector_grid")
        .num_columns(2)
        .spacing([20.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            let result = latch::latch(ui, "note", *note, |ui| {
                let mut finished = false;

                ui.label(t!("tab.inspector.single_note.x"));
                let response = ui.add(egui::DragValue::new(&mut note.x).speed(1));
                finished |= response.drag_stopped() || response.lost_focus();
                ui.end_row();

                ui.label(t!("tab.inspector.single_note.beat"));
                let response = ui.beat(&mut note.beat);
                finished |= response.drag_stopped() || response.lost_focus();
                ui.end_row();

                if let NoteKind::Hold { hold_beat } = note.kind {
                    ui.label(t!("tab.inspector.single_note.hold_beat"));

                    let mut bind = hold_beat;
                    let response = ui.beat(&mut bind);
                    finished |= response.drag_stopped() || response.lost_focus();
                    if bind != hold_beat {
                        note.kind = NoteKind::Hold { hold_beat: bind };
                    }

                    ui.end_row();
                }

                ui.label(t!("tab.inspector.single_note.above"));
                let response = ui.checkbox(&mut note.above, "");
                finished |= response.changed();
                ui.end_row();

                ui.label(t!("tab.inspector.single_note.speed"));
                let response = ui.add(egui::DragValue::new(&mut note.speed).speed(0.1));
                finished |= response.drag_stopped() || response.lost_focus();

                finished
            });

            if let Some(from) = result {
                if from != *note {
                    event_writer.send(DoCommandEvent(EditorCommand::EditNote(EditNote::new(
                        entity, from, *note,
                    ))));
                }
            }
        });
}

fn multiple_notes_inspector(
    ui: &mut Ui,
    notes: &[(Mut<Note>, Entity)],
    mut event_writer: EventWriter<DoCommandEvent>,
) {
    ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
        if ui
            .button(t!("tab.inspector.multiple_notes.flip_by_x"))
            .clicked()
        {
            let commands = notes
                .iter()
                .map(|(note, entity)| {
                    EditorCommand::EditNote(EditNote::new(
                        *entity,
                        **note,
                        Note {
                            x: -note.x,
                            ..**note
                        },
                    ))
                })
                .collect::<Vec<_>>();

            event_writer.send(DoCommandEvent(EditorCommand::CommandSequence(
                CommandSequence(commands),
            )));
        }
        if ui
            .button(t!("tab.inspector.multiple_notes.flip_by_selection"))
            .clicked()
        {
            let x_sum: f32 = notes.iter().map(|(note, _)| note.x).sum();
            let x_avg = x_sum / notes.len() as f32;

            let commands = notes
                .iter()
                .map(|(note, entity)| {
                    EditorCommand::EditNote(EditNote::new(
                        *entity,
                        **note,
                        Note {
                            x: 2.0 * x_avg - note.x,
                            ..**note
                        },
                    ))
                })
                .collect::<Vec<_>>();

            event_writer.send(DoCommandEvent(EditorCommand::CommandSequence(
                CommandSequence(commands),
            )));
        }

        let mut into_kind = |kind: NoteKind| {
            let commands = notes
                .iter()
                .map(|(note, entity)| {
                    EditorCommand::EditNote(EditNote::new(*entity, **note, Note { kind, ..**note }))
                })
                .collect::<Vec<_>>();

            event_writer.send(DoCommandEvent(EditorCommand::CommandSequence(
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
}

fn multiple_events_inspector(
    ui: &mut Ui,
    notes: &[(Mut<LineEvent>, Entity)],
    mut event_writer: EventWriter<DoCommandEvent>,
) {
    ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
        if ui
            .button(t!("tab.inspector.multiple_events.negate"))
            .clicked()
        {
            let commands = notes
                .iter()
                .filter(|(event, _)| event.kind != LineEventKind::Opacity)
                .map(|(event, entity)| {
                    EditorCommand::EditEvent(EditEvent::new(
                        *entity,
                        **event,
                        LineEvent {
                            start: -event.start,
                            end: -event.end,
                            ..**event
                        },
                    ))
                })
                .collect::<Vec<_>>();

            event_writer.send(DoCommandEvent(EditorCommand::CommandSequence(
                CommandSequence(commands),
            )));
        }
    });
}
