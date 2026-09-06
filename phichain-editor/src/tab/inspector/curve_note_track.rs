use crate::selection::Selected;
use crate::ui::widgets::easing::EasingValue;
use bevy::prelude::*;
use egui::{Color32, DragValue, RichText, Ui};
use phichain_chart::curve_note_track::CurveNoteTrackOptions;
use phichain_chart::note::NoteKind;
use phichain_game::curve_note_track::{CurveNoteTrackReadOnly, CurveNoteTrackTo};

fn options_grid(ui: &mut Ui, options: &mut CurveNoteTrackOptions) {
    egui::Grid::new("inspector_grid")
        .num_columns(2)
        .spacing([20.0, 2.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label(t!("tab.inspector.curve_note_track.density"));
            ui.add(DragValue::new(&mut options.density).range(1..=32).speed(1));
            ui.end_row();

            ui.label(t!("tab.inspector.curve_note_track.kind"));
            ui.horizontal(|ui| {
                ui.selectable_value(&mut options.kind, NoteKind::Tap, "Tap");
                ui.selectable_value(&mut options.kind, NoteKind::Drag, "Drag");
                ui.selectable_value(&mut options.kind, NoteKind::Flick, "Flick");
            });
            ui.end_row();

            ui.label(t!("tab.inspector.curve_note_track.curve"));
            ui.add(EasingValue::new(&mut options.curve));
            ui.end_row();
        });
}

pub fn curve_note_track_inspector(
    In(mut ui): In<Ui>,
    track: Single<CurveNoteTrackReadOnly, With<Selected>>,
) -> Result {
    ui.label(t!("tab.inspector.curve_note_track.title.selected"));
    ui.separator();

    options_grid(&mut ui, &mut track.options.clone());

    ui.separator();

    Ok(())
}

pub fn pending_curve_note_track_inspector(
    In(mut ui): In<Ui>,
    options: Single<&CurveNoteTrackOptions, (With<Selected>, Without<CurveNoteTrackTo>)>,
) -> Result {
    ui.label(t!("tab.inspector.curve_note_track.title.pending"));
    ui.separator();
    ui.label(
        RichText::new(t!(
            "tab.inspector.curve_note_track.instructions.select_destination"
        ))
        .color(Color32::RED),
    );
    ui.separator();

    options_grid(&mut ui, &mut (**options).clone());

    Ok(())
}
