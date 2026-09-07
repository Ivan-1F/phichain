use crate::editing::history::Edits;
use crate::selection::Selected;
use crate::timeline::TimelineContext;
use crate::ui::sides::SidesExt;
use crate::ui::widgets::beat_value::BeatValue;
use bevy::prelude::*;
use egui::{DragValue, Ui};
use phichain_chart::note::{Note, NoteKind};
use phichain_game::{Derived, Pending};

pub fn single_note_inspector(
    In(mut ui): In<Ui>,
    note: Single<(&Note, Entity, Has<Derived>, Has<Pending>), With<Selected>>,
    ctx: TimelineContext,
    mut edits: Edits,
) -> Result {
    let (note, entity, derived, pending) = note.into_inner();
    if derived || pending {
        ui.disable();
    }
    let mut display = *note;
    ui.label(t!("tab.inspector.single_note.title", kind = note.kind));
    ui.separator();

    ui.sides(
        |ui| ui.label(t!("tab.inspector.single_note.beat")),
        |ui| {
            ui.add_enabled(
                false,
                BeatValue::new(&mut display.beat)
                    .reversed(true)
                    .density(ctx.settings.density),
            )
        },
    );
    ui.sides(
        |ui| ui.label(t!("tab.inspector.single_note.x")),
        |ui| ui.add_enabled(false, DragValue::new(&mut display.x).speed(1)),
    );
    if let NoteKind::Hold { mut hold_beat } = display.kind {
        ui.sides(
            |ui| ui.label(t!("tab.inspector.single_note.hold_beat")),
            |ui| {
                ui.add_enabled(
                    false,
                    BeatValue::new(&mut hold_beat)
                        .reversed(true)
                        .density(ctx.settings.density),
                )
            },
        );
    }
    ui.sides(
        |ui| ui.label(t!("tab.inspector.single_note.above")),
        |ui| {
            let mut above = note.above;
            if ui.checkbox(&mut above, "").changed() {
                let next = Note { above, ..*note };
                edits.once(t!("history.edit_notes", count = 1), move |commands| {
                    commands.entity(entity).insert(next);
                });
            }
        },
    );
    ui.sides(
        |ui| ui.label(t!("tab.inspector.single_note.speed")),
        |ui| ui.add_enabled(false, DragValue::new(&mut display.speed).speed(0.1)),
    );
    Ok(())
}
