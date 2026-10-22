use crate::editing::history::{EditorHistory, HistoryEntry};
use bevy::prelude::*;
use egui::{RichText, ScrollArea, Sense, Ui, UiBuilder};

pub fn history_tab(In(mut ui): In<Ui>, history: Res<EditorHistory>, mut commands: Commands) {
    let position = history.position();
    let saved = history.saved_position();
    let entries: Vec<_> = history.entries().collect();

    if entries.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.weak(t!("tab.history.empty"));
        });
        return;
    }

    let mut jump_to = None;
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(&mut ui, |ui| {
            if clickable_row(ui, position == 0, |ui| {
                ui.label(RichText::new(t!("tab.history.open_project")).weak());
            })
            .clicked()
            {
                jump_to = Some(0);
            }
            for (index, entry) in entries.iter().enumerate() {
                if saved == Some(index) {
                    saved_marker(ui);
                }
                let target = index + 1;
                if clickable_row(ui, target == position, |ui| entry_row_contents(ui, entry))
                    .clicked()
                {
                    jump_to = Some(target);
                }
            }
            if saved == Some(entries.len()) {
                saved_marker(ui);
            }
        });

    if let Some(target) = jump_to {
        commands.queue(move |world: &mut World| {
            world.resource_scope(|world, mut history: Mut<EditorHistory>| {
                history.go_to(world, target);
            });
        });
    }
}

/// A full-width row that jumps to a history state on click.
fn clickable_row(ui: &mut Ui, current: bool, add_contents: impl FnOnce(&mut Ui)) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
        Sense::click(),
    );
    if response.hovered() {
        ui.painter()
            .rect_filled(rect, 2, ui.visuals().widgets.hovered.bg_fill);
    } else if current {
        ui.painter()
            .rect_filled(rect, 2, ui.visuals().faint_bg_color);
    }
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(rect.shrink2(egui::vec2(6.0, 0.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    // Keep the row the sole click target: selectable labels would steal clicks
    // over the text and show a text cursor.
    child.style_mut().interaction.selectable_labels = false;
    // Reserve the marker width on non-current rows to keep entries aligned.
    let marker = if current {
        RichText::new(egui_phosphor::regular::CARET_RIGHT)
    } else {
        RichText::new(egui_phosphor::regular::CARET_RIGHT).color(egui::Color32::TRANSPARENT)
    };
    child.label(marker);
    add_contents(&mut child);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn entry_row_contents(ui: &mut Ui, entry: &HistoryEntry) {
    let description = if entry.applied {
        RichText::new(entry.description)
    } else {
        RichText::new(entry.description).weak()
    };
    ui.add(egui::Label::new(description).truncate());
}

fn saved_marker(ui: &mut Ui) {
    ui.label(
        RichText::new(format!(
            "{} {}",
            egui_phosphor::regular::FLOPPY_DISK,
            t!("tab.history.saved")
        ))
        .small()
        .weak(),
    );
}
