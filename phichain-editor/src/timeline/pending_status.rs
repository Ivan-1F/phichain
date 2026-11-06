use crate::timeline::container::TimelineRect;
use crate::timeline::pending::PendingOf;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use egui::{RichText, Ui};

const CARD_MAX_WIDTH: f32 = 280.0;
const CARD_MARGIN: f32 = 12.0;

/// What the pending object is and how to finish it, shown on its owner
#[derive(Component, Debug, Clone)]
pub struct PendingStatus {
    pub title: String,
    pub hints: Vec<PendingHint>,
}

/// How to finish the pending object, optionally with the key to press
#[derive(Debug, Clone)]
pub struct PendingHint {
    pub key: Option<String>,
    pub action: String,
}

impl PendingHint {
    pub fn key(key: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            key: Some(key.into()),
            action: action.into(),
        }
    }

    pub fn text(action: impl Into<String>) -> Self {
        Self {
            key: None,
            action: action.into(),
        }
    }
}

/// Outline the owner of the pending object and show its status at the bottom
pub fn pending_status_ui(ui: &mut Ui, world: &mut World) {
    let mut state: SystemState<(
        Query<(Entity, &PendingOf, &PendingStatus)>,
        Query<&TimelineRect>,
    )> = SystemState::new(world);
    let (pending, rects) = state.get(world);
    let Some((entity, owner, status)) = pending.iter().next() else {
        return;
    };
    let Ok(&TimelineRect(viewport)) = rects.get(owner.0) else {
        return;
    };
    if !viewport.is_positive() {
        return;
    }
    let status = status.clone();
    let owner = owner.0;

    let outline = egui::Stroke::new(1.5_f32, ui.visuals().selection.bg_fill);
    ui.painter()
        .rect_stroke(viewport, 0.0, outline, egui::StrokeKind::Inside);

    let width = (viewport.width() - 2.0 * CARD_MARGIN).clamp(0.0, CARD_MAX_WIDTH);
    let cancel = egui::Area::new(egui::Id::new(("pending_status", owner)))
        .order(egui::Order::Foreground)
        .pivot(egui::Align2::CENTER_BOTTOM)
        .fixed_pos(egui::pos2(
            viewport.center().x,
            viewport.bottom() - CARD_MARGIN,
        ))
        .constrain_to(viewport)
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style())
                .show(ui, |ui| {
                    ui.set_width(width);
                    status_ui(ui, &status)
                })
                .inner
        })
        .inner;
    if cancel {
        world.despawn(entity);
    }
}

/// Returns whether the cancel button is clicked
fn status_ui(ui: &mut Ui, status: &PendingStatus) -> bool {
    let (_, cancel) = egui::Sides::new().shrink_left().show(
        ui,
        |ui| {
            ui.add(egui::Label::new(RichText::new(&status.title).strong()).truncate());
        },
        |ui| {
            ui.button(t!("tab.timeline.pending.cancel"))
                .on_hover_text("Esc")
                .clicked()
        },
    );

    ui.horizontal_wrapped(|ui| {
        for hint in &status.hints {
            if let Some(key) = &hint.key {
                ui.label(RichText::new(key).strong());
            }
            ui.label(&hint.action);
            ui.add_space(8.0);
        }
    });

    cancel
}
