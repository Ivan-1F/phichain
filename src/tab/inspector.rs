use bevy::prelude::*;
use egui::Ui;

use crate::chart::event::{LineEvent, LineEventKind};
use crate::chart::note::NoteKind;
use crate::editing::command::note::EditNote;
use crate::editing::command::EditorCommand;
use crate::editing::EditEvent;
use crate::ui::latch;
use crate::widgets::beat_value::BeatExt;
use crate::widgets::easing_value::EasingValue;
use crate::{chart::note::Note, selection::Selected};

pub fn inspector_ui_system(
    In(ui): In<&mut Ui>,
    mut selected_notes: Query<(&mut Note, Entity), With<Selected>>,
    mut selected_events: Query<&mut LineEvent, With<Selected>>,
    event_writer: EventWriter<EditEvent>,
) {
    let mut selected_notes: Vec<_> = selected_notes.iter_mut().collect();
    let mut selected_events: Vec<_> = selected_events.iter_mut().collect();
    if selected_notes.len() == 1 && selected_events.is_empty() {
        let (selected_note, entity) = selected_notes.get_mut(0).unwrap();
        single_note_inspector(ui, *entity, selected_note, event_writer);
    } else if selected_notes.is_empty() && selected_events.len() == 1 {
        let selected_event = selected_events.get_mut(0).unwrap();
        single_event_inspector(ui, selected_event, event_writer);
    }
}

fn single_event_inspector(
    ui: &mut Ui,
    event: &mut LineEvent,
    mut _event_writer: EventWriter<EditEvent>,
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
                ui.add(
                    egui::DragValue::new(&mut event.start)
                        .clamp_range(range.clone())
                        .speed(1.0),
                );
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
                    println!("{:?} -> {:?}", from, event);
                }
            }
        });
}

fn single_note_inspector(
    ui: &mut Ui,
    entity: Entity,
    note: &mut Note,
    mut event_writer: EventWriter<EditEvent>,
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
                    event_writer.send(EditEvent(EditorCommand::EditNote(EditNote::new(
                        entity, from, *note,
                    ))));
                }
            }
        });
}
