use crate::selection::Selected;
use crate::ui::widgets::easing::EasingValue;
use bevy::prelude::*;
use egui::{Color32, DragValue, RichText, Ui};
use phichain_chart::note::NoteKind;
use phichain_game::curve_note_track::CurveNoteTrack;

// TODO: write to history to support undo/redo
pub fn curve_note_track_inspector(
    In(mut ui): In<Ui>,
    mut track: Single<&mut CurveNoteTrack, With<Selected>>,
) -> Result {
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
        .show(&mut ui, |ui| {
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
            ui.add(EasingValue::new(&mut track.options.curve));
            ui.end_row();
        });

    ui.separator();

    Ok(())
}
