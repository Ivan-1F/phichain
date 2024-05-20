use bevy::prelude::*;
use egui::Ui;

use crate::chart::event::{LineEvent, LineEventKind};
use crate::widgets::beat_value::BeatExt;
use crate::widgets::easing_value::EasingValue;
use crate::{
    chart::note::{Note, NoteKind},
    selection::Selected,
};

pub fn inspector_ui_system(
    In(ui): In<&mut Ui>,
    mut selected_notes: Query<&mut Note, With<Selected>>,
    mut selected_events: Query<&mut LineEvent, With<Selected>>,
) {
    let mut selected_notes: Vec<_> = selected_notes.iter_mut().collect();
    let mut selected_events: Vec<_> = selected_events.iter_mut().collect();
    if selected_notes.len() == 1 && selected_events.is_empty() {
        let selected_note = selected_notes.get_mut(0).unwrap();
        single_note_inspector(ui, selected_note);
    } else if selected_notes.is_empty() && selected_events.len() == 1 {
        let selected_event = selected_events.get_mut(0).unwrap();
        single_event_inspector(ui, selected_event);
    }
}

fn single_event_inspector(ui: &mut Ui, event: &mut LineEvent) {
    egui::Grid::new("inspector_grid")
        .num_columns(2)
        .spacing([20.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label(t!("tab.inspector.single_event.start_beat"));
            ui.beat(&mut event.start_beat);
            ui.end_row();

            ui.label(t!("tab.inspector.single_event.end_beat"));
            ui.beat(&mut event.end_beat);
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
            ui.add(
                egui::DragValue::new(&mut event.end)
                    .clamp_range(range.clone())
                    .speed(1.0),
            );
            ui.end_row();

            ui.label(t!("tab.inspector.single_event.end_value"));
            ui.add(EasingValue::new(&mut event.easing));
            ui.end_row();
        });
}

fn single_note_inspector(ui: &mut Ui, note: &mut Note) {
    egui::Grid::new("inspector_grid")
        .num_columns(2)
        .spacing([20.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label(t!("tab.inspector.single_note.x"));
            ui.add(egui::DragValue::new(&mut note.x).speed(1));
            ui.end_row();

            ui.label(t!("tab.inspector.single_note.beat"));
            ui.beat(&mut note.beat);
            ui.end_row();

            if let NoteKind::Hold { hold_beat } = note.kind {
                ui.label(t!("tab.inspector.single_note.hold_beat"));

                let mut bind = hold_beat;
                ui.beat(&mut bind);
                if bind != hold_beat {
                    note.kind = NoteKind::Hold { hold_beat: bind };
                }

                ui.end_row();
            }

            ui.label(t!("tab.inspector.single_note.above"));
            ui.checkbox(&mut note.above, "");
        });
}
