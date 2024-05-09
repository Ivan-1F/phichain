use bevy::prelude::*;
use egui::Ui;
use fraction::Fraction;

use crate::{chart::{beat::Beat, note::Note}, selection::Selected};

pub fn inspector_ui_system(In(ui): In<&mut Ui>, mut selected_notes: Query<&mut Note, With<Selected>>) {
    let mut selected_notes: Vec<_> = selected_notes.iter_mut().collect();
    if selected_notes.len() == 1 {
        let selected_note = selected_notes.get_mut(0).unwrap();
        ui.add(egui::DragValue::new(&mut selected_note.x).speed(0.01));
        ui.horizontal(|ui| {
            let mut beat = selected_note.beat.beat();
            let mut numer = selected_note.beat.numer();
            let mut denom = selected_note.beat.denom();
            ui.add(egui::DragValue::new(&mut beat).clamp_range(0..=u32::MAX).speed(1));
            ui.add(egui::DragValue::new(&mut numer).clamp_range(0..=u32::MAX).speed(1));
            ui.add(egui::DragValue::new(&mut denom).clamp_range(1..=u32::MAX).speed(1));

            if beat != selected_note.beat.beat() || numer != selected_note.beat.numer() || denom != selected_note.beat.denom() {
                selected_note.beat = Beat::from(Fraction::new(beat * denom + numer, denom));
            }
        });
    }
}
