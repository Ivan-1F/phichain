use crate::editing::history::Edits;
use crate::selection::Selected;
use crate::timeline::TimelineContext;
use crate::ui::widgets::beat_value::BeatValue;
use bevy::prelude::*;
use egui::{DragValue, Ui};
use phichain_chart::note::Note;
use phichain_game::{Derived, Pending};

pub fn single_note_inspector(
    In(mut ui): In<Ui>,
    selected: Single<(&Note, Entity, Has<Derived>, Has<Pending>), With<Selected>>,
    ctx: TimelineContext,
    mut edits: Edits,
) -> Result {
    let (note, entity, derived, pending) = selected.into_inner();
    ui.label(t!("tab.inspector.single_note.title", kind = note.kind));
    ui.separator();

    ui.add_enabled_ui(!derived && !pending, |ui| {
        let mut editor = edits.component(entity, note, t!("history.edit_notes", count = 1));
        editor.field(
            ui,
            "beat",
            t!("tab.inspector.single_note.beat"),
            |ui, note| {
                ui.add(
                    BeatValue::new(&mut note.beat)
                        .reversed(true)
                        .density(ctx.settings.density),
                );
            },
        );
        editor.field(ui, "x", t!("tab.inspector.single_note.x"), |ui, note| {
            ui.add(DragValue::new(&mut note.x).speed(1));
        });
        if note.kind.is_hold() {
            editor.field(
                ui,
                "hold_beat",
                t!("tab.inspector.single_note.hold_beat"),
                |ui, note| {
                    ui.add(
                        BeatValue::new(note.hold_beat_mut().unwrap())
                            .reversed(true)
                            .density(ctx.settings.density),
                    );
                },
            );
        }
        editor.field(
            ui,
            "above",
            t!("tab.inspector.single_note.above"),
            |ui, note| {
                ui.checkbox(&mut note.above, "");
            },
        );
        editor.field(
            ui,
            "speed",
            t!("tab.inspector.single_note.speed"),
            |ui, note| {
                ui.add(DragValue::new(&mut note.speed).speed(0.1));
            },
        );
    });
    Ok(())
}
