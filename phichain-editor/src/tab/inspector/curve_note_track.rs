use crate::editing::history::Edits;
use crate::selection::Selected;
use crate::ui::edit::EditResponse;
use crate::ui::widgets::easing::{EasingParameter, EasingValue};
use bevy::prelude::*;
use egui::{DragValue, Ui};
use phichain_chart::note::NoteKind;
use phichain_game::curve_note_track::CurveNoteTrack;

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

fn note_kind_ui(ui: &mut Ui, kind: &mut NoteKind) -> egui::Response {
    ui.horizontal(|ui| {
        ui.selectable_value(kind, NoteKind::Tap, "Tap")
            | ui.selectable_value(kind, NoteKind::Drag, "Drag")
            | ui.selectable_value(kind, NoteKind::Flick, "Flick")
    })
    .inner
}
