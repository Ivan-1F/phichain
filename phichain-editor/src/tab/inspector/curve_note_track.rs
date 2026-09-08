use crate::editing::history::Edits;
use crate::selection::Selected;
use crate::ui::edit::EditResponse;
use crate::ui::widgets::easing::{EasingParameter, EasingValue};
use bevy::prelude::*;
use egui::{Color32, DragValue, RichText, Ui};
use phichain_chart::curve_note_track::CurveNoteTrackOptions;
use phichain_chart::note::NoteKind;
use phichain_game::curve_note_track::{CurveNoteTrack, CurveNoteTrackTo};

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
            note_kind_ui(ui, &mut options.kind);
            ui.end_row();

            ui.label(t!("tab.inspector.curve_note_track.curve"));
            ui.add(EasingValue::new(&mut options.curve));
            ui.end_row();
            if options.curve.is_steps() || options.curve.is_elastic() {
                ui.label(t!("game.easing.parameter"));
                ui.add(EasingParameter(&mut options.curve));
                ui.end_row();
            }
        });
}

pub fn curve_note_track_inspector(
    In(mut ui): In<Ui>,
    track: Single<(Entity, CurveNoteTrack), With<Selected>>,
    mut edits: Edits,
) -> Result {
    ui.label(t!("tab.inspector.curve_note_track.title.selected"));
    ui.separator();

    let (entity, track) = track.into_inner();
    let mut editor = edits.component(entity, track.options, t!("history.edit_tracks", count = 1));
    editor.field(
        &mut ui,
        "density",
        t!("tab.inspector.curve_note_track.density"),
        |ui, options| {
            ui.add(DragValue::new(&mut options.density).range(1..=32).speed(1));
        },
    );
    editor.field(
        &mut ui,
        "kind",
        t!("tab.inspector.curve_note_track.kind"),
        |ui, options| {
            ui.custom(|ui| EditResponse::Once(note_kind_ui(ui, &mut options.kind)));
        },
    );
    editor.field(
        &mut ui,
        "curve",
        t!("tab.inspector.curve_note_track.curve"),
        |ui, options| {
            ui.add(EasingValue::new(&mut options.curve));
        },
    );
    if editor.value().curve.is_steps() || editor.value().curve.is_elastic() {
        editor.field(
            &mut ui,
            "parameter",
            t!("game.easing.parameter"),
            |ui, options| {
                ui.add(EasingParameter(&mut options.curve));
            },
        );
    }

    ui.separator();

    Ok(())
}

pub fn pending_curve_note_track_inspector(
    In(mut ui): In<Ui>,
    options: Single<(Entity, &CurveNoteTrackOptions), (With<Selected>, Without<CurveNoteTrackTo>)>,
    mut commands: Commands,
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

    let (entity, original) = options.into_inner();
    let mut options = original.clone();
    options_grid(&mut ui, &mut options);
    if options != *original {
        commands.entity(entity).insert(options);
    }

    Ok(())
}

fn note_kind_ui(ui: &mut Ui, kind: &mut NoteKind) -> egui::Response {
    ui.horizontal(|ui| {
        ui.selectable_value(kind, NoteKind::Tap, "Tap")
            | ui.selectable_value(kind, NoteKind::Drag, "Drag")
            | ui.selectable_value(kind, NoteKind::Flick, "Flick")
    })
    .inner
}
