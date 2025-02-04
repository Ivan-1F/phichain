use crate::editing::command::event::EditEvent;
use crate::editing::command::note::EditNote;
use crate::editing::command::{CommandSequence, EditorCommand};
use crate::editing::DoCommandEvent;
use crate::selection::{Selected, SelectedLine};
use crate::ui::latch;
use crate::ui::widgets::beat_value::BeatExt;
use crate::ui::widgets::easing::EasingValue;
use bevy::prelude::*;
use egui::{Align, Color32, DragValue, Layout, RichText, Ui};
use phichain_chart::beat;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
use phichain_chart::line::Line;
use phichain_chart::note::{Note, NoteKind};
use phichain_game::curve_note_track::CurveNoteTrack;

pub fn inspector_ui_system(
    In(mut ui): In<Ui>,

    mut selected_notes: Query<(&mut Note, Entity), With<Selected>>,
    mut selected_events: Query<(&mut LineEvent, Entity), With<Selected>>,
    selected_line: Res<SelectedLine>,
    mut line_query: Query<&mut Line>,
    event_writer: EventWriter<DoCommandEvent>,

    mut selected_track: Query<&mut CurveNoteTrack, With<Selected>>,
) {
    if let Ok(mut track) = selected_track.get_single_mut() {
        curve_note_track_inspector(&mut ui, &mut track);
        return;
    }

    let mut selected_notes: Vec<_> = selected_notes.iter_mut().collect();
    let mut selected_events: Vec<_> = selected_events.iter_mut().collect();
    if selected_notes.len() == 1 && selected_events.is_empty() {
        let (selected_note, entity) = selected_notes.get_mut(0).unwrap();
        single_note_inspector(&mut ui, *entity, selected_note, event_writer);
    } else if selected_notes.is_empty() && selected_events.len() == 1 {
        let (selected_event, entity) = selected_events.get_mut(0).unwrap();
        single_event_inspector(&mut ui, *entity, selected_event, event_writer);
    } else if selected_notes.len() > 1 && selected_events.is_empty() {
        multiple_notes_inspector(&mut ui, &selected_notes, event_writer);
    } else if selected_notes.is_empty() && selected_events.len() > 1 {
        multiple_events_inspector(&mut ui, &selected_events, event_writer);
    } else if let Ok(mut line) = line_query.get_mut(selected_line.0) {
        line_inspector(&mut ui, &mut line);
    }
}

fn curve_note_track_inspector(ui: &mut Ui, track: &mut CurveNoteTrack) {
    match (track.from.is_some(), track.to.is_some()) {
        (true, true) => {}
        (true, false) => {
            ui.label(
                RichText::new(t!(
                    "tab.inspector.curve_note_track.instructions.select_destination"
                ))
                .color(Color32::RED),
            );
            ui.separator();
        }
        (false, true) => {
            ui.label(
                RichText::new(t!(
                    "tab.inspector.curve_note_track.instructions.select_origin"
                ))
                .color(Color32::RED),
            );
            ui.separator();
        }
        (false, false) => {
            ui.label(
                RichText::new(t!(
                    "tab.inspector.curve_note_track.instructions.select_origin_destination"
                ))
                .color(Color32::RED),
            );
            ui.separator();
        }
    }

    egui::Grid::new("inspector_grid")
        .num_columns(2)
        .spacing([20.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label(t!("tab.inspector.curve_note_track.density"));
            ui.add(
                DragValue::new(&mut track.options.density)
                    .range(1..=32)
                    .speed(1),
            );
            ui.end_row();

            ui.label(t!("tab.inspector.curve_note_track.kind"));
            ui.horizontal(|ui| {
                ui.selectable_value(&mut track.options.kind, NoteKind::Tap, "Tap");
                ui.selectable_value(&mut track.options.kind, NoteKind::Drag, "Drag");
                ui.selectable_value(&mut track.options.kind, NoteKind::Flick, "Flick");
            });
            ui.end_row();

            ui.label(t!("tab.inspector.curve_note_track.curve"));
            ui.add(EasingValue::new(&mut track.options.curve).show_graph(false));
            ui.end_row();
        });

    ui.separator();
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

                ui.label(t!("tab.inspector.single_event.value_type"));
                ui.columns(2, |columns| {
                    if columns[0]
                        .selectable_label(
                            event.value.is_transition(),
                            t!("tab.inspector.single_event.transition"),
                        )
                        .clicked()
                    {
                        let mut new_event = *event;
                        new_event.value = new_event.value.into_transition();
                        event_writer.send(DoCommandEvent(EditorCommand::EditEvent(
                            EditEvent::new(entity, *event, new_event),
                        )));
                    }
                    if columns[1]
                        .selectable_label(
                            event.value.is_constant(),
                            t!("tab.inspector.single_event.constant"),
                        )
                        .clicked()
                    {
                        let mut new_event = *event;
                        new_event.value = new_event.value.into_constant();
                        event_writer.send(DoCommandEvent(EditorCommand::EditEvent(
                            EditEvent::new(entity, *event, new_event),
                        )));
                    }
                });
                ui.end_row();

                match event.value {
                    LineEventValue::Transition {
                        ref mut start,
                        ref mut end,
                        ref mut easing,
                    } => {
                        ui.label(t!("tab.inspector.single_event.start_value"));
                        let range = match event.kind {
                            LineEventKind::Opacity => 0.0..=255.0,
                            _ => f32::MIN..=f32::MAX,
                        };
                        let response =
                            ui.add(egui::DragValue::new(start).range(range.clone()).speed(1.0));
                        finished |= response.drag_stopped() || response.lost_focus();
                        ui.end_row();

                        ui.label(t!("tab.inspector.single_event.end_value"));
                        let response =
                            ui.add(egui::DragValue::new(end).range(range.clone()).speed(1.0));
                        finished |= response.drag_stopped() || response.lost_focus();
                        ui.end_row();

                        if !event.kind.is_speed() {
                            ui.label(t!("tab.inspector.single_event.easing"));
                            let response = ui.add(EasingValue::new(easing));
                            finished |= response.drag_stopped() || response.lost_focus();
                            ui.end_row();
                        }
                    }
                    LineEventValue::Constant(ref mut value) => {
                        ui.label(t!("tab.inspector.single_event.value"));
                        let range = match event.kind {
                            LineEventKind::Opacity => 0.0..=255.0,
                            _ => f32::MIN..=f32::MAX,
                        };
                        let response =
                            ui.add(egui::DragValue::new(value).range(range.clone()).speed(1.0));
                        finished |= response.drag_stopped() || response.lost_focus();
                        ui.end_row();
                    }
                }

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

        if ui
            .button(t!("tab.inspector.multiple_notes.flip_side"))
            .clicked()
        {
            let commands = notes
                .iter()
                .map(|(note, entity)| {
                    EditorCommand::EditNote(EditNote::new(
                        *entity,
                        **note,
                        Note {
                            above: !note.above,
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
                            value: event.value.negated(),
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

fn line_inspector(ui: &mut Ui, line: &mut Line) {
    egui::Grid::new("inspector_grid")
        .num_columns(2)
        .spacing([20.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            let result = latch::latch(ui, "line", line.clone(), |ui| {
                let mut finished = false;

                ui.label(t!("tab.inspector.line.name"));
                let response = ui.text_edit_singleline(&mut line.name);
                finished |= response.lost_focus();
                ui.end_row();

                finished
            });

            if let Some(from) = result {
                if from != line.clone() {
                    // TODO: write to history to support undo/redo
                }
            }
        });
}
