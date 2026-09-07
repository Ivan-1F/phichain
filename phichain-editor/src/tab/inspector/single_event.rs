use crate::editing::history::Edits;
use crate::selection::Selected;
use crate::timeline::TimelineContext;
use crate::ui::sides::SidesExt;
use crate::ui::widgets::beat_value::BeatValue;
use crate::ui::widgets::easing::{EasingGraph, EasingValue};
use bevy::prelude::*;
use egui::{DragValue, Ui};
use phichain_chart::beat::Beat;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};

pub fn single_event_inspector(
    In(mut ui): In<Ui>,
    event: Single<(&LineEvent, Entity), With<Selected>>,
    ctx: TimelineContext,
    mut edits: Edits,
) -> Result {
    let (original, entity) = event.into_inner();
    let mut draft = *original;
    let event = &mut draft;

    let kind = match event.kind {
        LineEventKind::X => t!("game.event.kind.x"),
        LineEventKind::Y => t!("game.event.kind.y"),
        LineEventKind::Rotation => t!("game.event.kind.rotation"),
        LineEventKind::Opacity => t!("game.event.kind.opacity"),
        LineEventKind::Speed => t!("game.event.kind.speed"),
    };

    ui.label(t!("tab.inspector.single_event.title", kind = kind));
    ui.separator();

    ui.sides(
        |ui| ui.label(t!("tab.inspector.single_event.start_beat")),
        |ui| {
            ui.add_enabled(
                false,
                BeatValue::new(&mut event.start_beat)
                    .range(Beat::MIN..=event.end_beat)
                    .reversed(true)
                    .density(ctx.settings.density),
            );
        },
    );
    ui.sides(
        |ui| ui.label(t!("tab.inspector.single_event.end_beat")),
        |ui| {
            ui.add_enabled(
                false,
                BeatValue::new(&mut event.end_beat)
                    .range(event.start_beat..=Beat::MAX)
                    .reversed(true)
                    .density(ctx.settings.density),
            );
        },
    );
    ui.sides(
        |ui| ui.label(t!("tab.inspector.single_event.value_type")),
        |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(
                        event.value.is_transition(),
                        t!("tab.inspector.single_event.transition"),
                    )
                    .clicked()
                {
                    event.value = event.value.into_transition();
                }
                if ui
                    .selectable_label(
                        event.value.is_constant(),
                        t!("tab.inspector.single_event.constant"),
                    )
                    .clicked()
                {
                    event.value = event.value.into_constant();
                }
            });
        },
    );

    match event.value {
        LineEventValue::Transition {
            ref mut start,
            ref mut end,
            ref mut easing,
        } => {
            let range = match event.kind {
                LineEventKind::Opacity => 0.0..=255.0,
                _ => f32::MIN..=f32::MAX,
            };
            ui.sides(
                |ui| ui.label(t!("tab.inspector.single_event.start_value")),
                |ui| {
                    ui.add_enabled(false, DragValue::new(start).range(range.clone()).speed(1.0));
                },
            );
            ui.sides(
                |ui| ui.label(t!("tab.inspector.single_event.end_value")),
                |ui| {
                    ui.add_enabled(false, DragValue::new(end).range(range.clone()).speed(1.0));
                },
            );
            ui.sides(
                |ui| ui.label(t!("tab.inspector.single_event.easing")),
                |ui| {
                    ui.add(EasingValue::new(easing).numeric_editable(false));
                },
            );
            ui.separator();
            ui.add_enabled_ui(false, |ui| {
                ui.add_sized(
                    egui::Vec2::new(ui.available_width(), ui.available_width() / 3.0 * 2.0),
                    EasingGraph::new(easing),
                );
            });
        }
        LineEventValue::Constant { ref mut value } => {
            let range = match event.kind {
                LineEventKind::Opacity => 0.0..=255.0,
                _ => f32::MIN..=f32::MAX,
            };
            ui.sides(
                |ui| ui.label(t!("tab.inspector.single_event.value")),
                |ui| {
                    ui.add_enabled(false, DragValue::new(value).range(range.clone()).speed(1.0));
                },
            );
        }
    }

    if draft != *original {
        edits.once(t!("history.edit_events", count = 1), move |commands| {
            commands.entity(entity).insert(draft);
        });
    }
    Ok(())
}
