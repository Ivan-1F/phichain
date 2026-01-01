use crate::editing::command::event::EditEvent;
use crate::editing::command::EditorCommand;
use crate::editing::DoCommandEvent;
use crate::selection::Selected;
use crate::ui::latch;
use crate::ui::sides::SidesExt;
use crate::ui::widgets::beat_value::BeatValue;
use crate::ui::widgets::easing::{EasingGraph, EasingValue};
use bevy::prelude::*;
use egui::{DragValue, Ui};
use phichain_chart::beat::Beat;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};

pub fn single_event_inspector(
    In(mut ui): In<Ui>,
    event: Single<(&mut LineEvent, Entity), With<Selected>>,
    mut event_writer: EventWriter<DoCommandEvent>,
) -> Result {
    let (mut event, entity) = event.into_inner();
    let event = event.as_mut();

    let kind = match event.kind {
        LineEventKind::X => t!("game.event.kind.x"),
        LineEventKind::Y => t!("game.event.kind.y"),
        LineEventKind::Rotation => t!("game.event.kind.rotation"),
        LineEventKind::Opacity => t!("game.event.kind.opacity"),
        LineEventKind::Speed => t!("game.event.kind.speed"),
    };

    ui.label(t!("tab.inspector.single_event.title", kind = kind));
    ui.separator();

    let result = latch::latch(&mut ui, "event", *event, |ui| {
        let mut finished = false;

        ui.sides(
            |ui| ui.label(t!("tab.inspector.single_event.start_beat")),
            |ui| {
                let response = ui.add(
                    BeatValue::new(&mut event.start_beat)
                        .range(Beat::MIN..=event.end_beat)
                        .reversed(true),
                );
                finished |= response.drag_stopped() || response.lost_focus();
            },
        );
        ui.sides(
            |ui| ui.label(t!("tab.inspector.single_event.end_beat")),
            |ui| {
                let response = ui.add(
                    BeatValue::new(&mut event.end_beat)
                        .range(event.start_beat..=Beat::MAX)
                        .reversed(true),
                );
                finished |= response.drag_stopped() || response.lost_focus();
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
                        let mut new_event = *event;
                        new_event.value = new_event.value.into_transition();
                        event_writer.write(DoCommandEvent(EditorCommand::EditEvent(
                            EditEvent::new(entity, *event, new_event),
                        )));
                    }
                    if ui
                        .selectable_label(
                            event.value.is_constant(),
                            t!("tab.inspector.single_event.constant"),
                        )
                        .clicked()
                    {
                        let mut new_event = *event;
                        new_event.value = new_event.value.into_constant();
                        event_writer.write(DoCommandEvent(EditorCommand::EditEvent(
                            EditEvent::new(entity, *event, new_event),
                        )));
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
                        let response =
                            ui.add(DragValue::new(start).range(range.clone()).speed(1.0));
                        finished |= response.drag_stopped() || response.lost_focus();
                    },
                );
                ui.sides(
                    |ui| ui.label(t!("tab.inspector.single_event.end_value")),
                    |ui| {
                        let response = ui.add(DragValue::new(end).range(range.clone()).speed(1.0));
                        finished |= response.drag_stopped() || response.lost_focus();
                    },
                );
                ui.sides(
                    |ui| ui.label(t!("tab.inspector.single_event.easing")),
                    |ui| {
                        let response = ui.add(EasingValue::new(easing));
                        finished |= response.drag_stopped() || response.lost_focus();
                    },
                );
                ui.separator();
                let response = ui.add_sized(
                    egui::Vec2::new(ui.available_width(), ui.available_width() / 3.0 * 2.0),
                    EasingGraph::new(easing),
                );
                finished |= response.drag_stopped();
            }
            LineEventValue::Constant(ref mut value) => {
                let range = match event.kind {
                    LineEventKind::Opacity => 0.0..=255.0,
                    _ => f32::MIN..=f32::MAX,
                };
                ui.sides(
                    |ui| ui.label(t!("tab.inspector.single_event.value")),
                    |ui| {
                        let response =
                            ui.add(DragValue::new(value).range(range.clone()).speed(1.0));
                        finished |= response.drag_stopped() || response.lost_focus();
                    },
                );
            }
        }

        finished
    });

    if let Some(from) = result {
        if from != *event {
            event_writer.write(DoCommandEvent(EditorCommand::EditEvent(EditEvent::new(
                entity, from, *event,
            ))));
        }
    }

    Ok(())
}
