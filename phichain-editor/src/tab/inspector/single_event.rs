use crate::editing::history::Edits;
use crate::selection::Selected;
use crate::timeline::TimelineContext;
use crate::ui::edit::EditResponse;
use crate::ui::widgets::beat_value::BeatValue;
use crate::ui::widgets::easing::{EasingGraph, EasingParameter, EasingValue};
use bevy::prelude::*;
use egui::{DragValue, Ui};
use phichain_chart::beat::Beat;
use phichain_chart::event::{LineEvent, LineEventKind};
use phichain_game::Pending;

pub fn single_event_inspector(
    In(mut ui): In<Ui>,
    event: Single<(&LineEvent, Entity, Has<Pending>), With<Selected>>,
    ctx: TimelineContext,
    mut edits: Edits,
) -> Result {
    let (event, entity, pending) = event.into_inner();
    if pending {
        ui.disable();
    }
    let kind = match event.kind {
        LineEventKind::X => t!("game.event.kind.x"),
        LineEventKind::Y => t!("game.event.kind.y"),
        LineEventKind::Rotation => t!("game.event.kind.rotation"),
        LineEventKind::Opacity => t!("game.event.kind.opacity"),
        LineEventKind::Speed => t!("game.event.kind.speed"),
    };
    ui.label(t!("tab.inspector.single_event.title", kind = kind));
    ui.separator();

    let range = match event.kind {
        LineEventKind::Opacity => 0.0..=255.0,
        _ => f32::MIN..=f32::MAX,
    };
    let mut editor = edits.component(entity, event, t!("history.edit_events", count = 1));
    editor.field(
        &mut ui,
        "start_beat",
        t!("tab.inspector.single_event.start_beat"),
        |ui, event| {
            ui.add(
                BeatValue::new(&mut event.start_beat)
                    .range(Beat::MIN..=event.end_beat)
                    .reversed(true)
                    .density(ctx.settings.density),
            );
        },
    );
    editor.field(
        &mut ui,
        "end_beat",
        t!("tab.inspector.single_event.end_beat"),
        |ui, event| {
            ui.add(
                BeatValue::new(&mut event.end_beat)
                    .range(event.start_beat..=Beat::MAX)
                    .reversed(true)
                    .density(ctx.settings.density),
            );
        },
    );
    editor.field(
        &mut ui,
        "value_type",
        t!("tab.inspector.single_event.value_type"),
        |ui, event| {
            ui.custom(|ui| {
                let mut transition = event.value.is_transition();
                let response = ui
                    .horizontal(|ui| {
                        ui.selectable_value(
                            &mut transition,
                            true,
                            t!("tab.inspector.single_event.transition"),
                        ) | ui.selectable_value(
                            &mut transition,
                            false,
                            t!("tab.inspector.single_event.constant"),
                        )
                    })
                    .inner;
                if response.changed() {
                    event.value = if transition {
                        event.value.into_transition()
                    } else {
                        event.value.into_constant()
                    };
                }
                EditResponse::Once(response)
            });
        },
    );

    if editor.value().value.is_transition() {
        editor.field(
            &mut ui,
            "start_value",
            t!("tab.inspector.single_event.start_value"),
            |ui, event| {
                ui.add(
                    DragValue::new(event.value.start_mut())
                        .range(range.clone())
                        .speed(1.0),
                );
            },
        );
        editor.field(
            &mut ui,
            "end_value",
            t!("tab.inspector.single_event.end_value"),
            |ui, event| {
                ui.add(
                    DragValue::new(event.value.end_mut())
                        .range(range.clone())
                        .speed(1.0),
                );
            },
        );
        editor.field(
            &mut ui,
            "easing",
            t!("tab.inspector.single_event.easing"),
            |ui, event| {
                ui.add(EasingValue::new(event.value.easing_mut().unwrap()));
            },
        );
        let easing = editor.value().value.easing();
        if easing.is_steps() || easing.is_elastic() {
            editor.field(
                &mut ui,
                "easing_parameter",
                t!("game.easing.parameter"),
                |ui, event| {
                    ui.add(EasingParameter(event.value.easing_mut().unwrap()));
                },
            );
        }
        ui.separator();
        editor.edit(&mut ui, "easing_graph", |ui, event| {
            ui.add(EasingGraph::new(event.value.easing_mut().unwrap()));
        });
    } else {
        editor.field(
            &mut ui,
            "value",
            t!("tab.inspector.single_event.value"),
            |ui, event| {
                ui.add(
                    DragValue::new(event.value.start_mut())
                        .range(range)
                        .speed(1.0),
                );
            },
        );
    }
    Ok(())
}
