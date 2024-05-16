use bevy::prelude::*;
use egui::Ui;
use num::Rational32;

use crate::chart::event::{LineEvent, LineEventKind};
use crate::{
    chart::{
        beat::Beat,
        note::{Note, NoteKind},
    },
    selection::Selected,
    translation::Translator,
};

pub fn inspector_ui_system(
    In(ui): In<&mut Ui>,
    mut selected_notes: Query<&mut Note, With<Selected>>,
    mut selected_events: Query<&mut LineEvent, With<Selected>>,
    translator: Translator,
) {
    let mut selected_notes: Vec<_> = selected_notes.iter_mut().collect();
    let mut selected_events: Vec<_> = selected_events.iter_mut().collect();
    if selected_notes.len() == 1 && selected_events.is_empty() {
        let selected_note = selected_notes.get_mut(0).unwrap();
        single_note_inspector(ui, selected_note, &translator);
    } else if selected_notes.is_empty() && selected_events.len() == 1 {
        let selected_event = selected_events.get_mut(0).unwrap();
        single_event_inspector(ui, selected_event, &translator);
    }
}

trait BeatExt {
    fn beat(&mut self, beat: &mut Beat);
}

impl BeatExt for Ui {
    fn beat(&mut self, beat: &mut Beat) {
        self.horizontal(|ui| {
            let mut whole = beat.beat();
            let mut numer = beat.numer();
            let mut denom = beat.denom();
            ui.add(
                egui::DragValue::new(&mut whole)
                    .clamp_range(0..=u32::MAX)
                    .speed(1),
            );
            ui.add(
                egui::DragValue::new(&mut numer)
                    .clamp_range(0..=u32::MAX)
                    .speed(1),
            );
            ui.add(
                egui::DragValue::new(&mut denom)
                    .clamp_range(1..=u32::MAX)
                    .speed(1),
            );

            if whole != beat.beat() || numer != beat.numer() || denom != beat.denom() {
                *beat = Beat::new(whole, Rational32::new(numer, denom));
            }
        });
    }
}

fn single_event_inspector(ui: &mut Ui, event: &mut LineEvent, translator: &Translator) {
    egui::Grid::new("inspector_grid")
        .num_columns(2)
        .spacing([40.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label(translator.tr("tab.inspector.single_event.start_beat"));
            ui.beat(&mut event.start_beat);
            ui.end_row();

            ui.label(translator.tr("tab.inspector.single_event.end_beat"));
            ui.beat(&mut event.end_beat);
            ui.end_row();

            ui.label(translator.tr("tab.inspector.single_event.start_value"));
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

            ui.label(translator.tr("tab.inspector.single_event.end_value"));
            ui.add(
                egui::DragValue::new(&mut event.end)
                    .clamp_range(range.clone())
                    .speed(1.0),
            );
            ui.end_row();
        });
}

fn single_note_inspector(ui: &mut Ui, note: &mut Note, translator: &Translator) {
    egui::Grid::new("inspector_grid")
        .num_columns(2)
        .spacing([40.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label(translator.tr("tab.inspector.single_note.x"));
            ui.add(egui::DragValue::new(&mut note.x).speed(1));
            ui.end_row();

            ui.label(translator.tr("tab.inspector.single_note.beat"));
            ui.beat(&mut note.beat);
            ui.end_row();

            if let NoteKind::Hold { hold_beat } = note.kind {
                ui.label(translator.tr("tab.inspector.single_note.hold_beat"));

                let mut bind = hold_beat;
                ui.beat(&mut bind);
                if bind != hold_beat {
                    note.kind = NoteKind::Hold { hold_beat: bind };
                }

                ui.end_row();
            }

            ui.label(translator.tr("tab.inspector.single_note.above"));
            ui.checkbox(&mut note.above, "");
        });
}
