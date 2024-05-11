use bevy::prelude::*;
use egui::Ui;
use num::Rational32;

use crate::{
    chart::{beat::Beat, note::{Note, NoteKind}},
    selection::Selected,
};

pub fn inspector_ui_system(
    In(ui): In<&mut Ui>,
    mut selected_notes: Query<&mut Note, With<Selected>>,
) {
    let mut selected_notes: Vec<_> = selected_notes.iter_mut().collect();
    if selected_notes.len() == 1 {
        let selected_note = selected_notes.get_mut(0).unwrap();
        single_note_inspector(ui, selected_note);
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

fn single_note_inspector(ui: &mut Ui, note: &mut Note) {
    egui::Grid::new("inspector_grid")
        .num_columns(2)
        .spacing([40.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("X");
            ui.add(egui::DragValue::new(&mut note.x).speed(1));
            ui.end_row();

            ui.label("Beat");
            ui.beat(&mut note.beat);
            ui.end_row();

            if let NoteKind::Hold { mut hold_beat } = note.kind {
                ui.label("Hold Beat");
                ui.beat(&mut hold_beat);
                ui.end_row();
            }
            
            ui.label("Above");
            ui.checkbox(&mut note.above, "");
        });
}
