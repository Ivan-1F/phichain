use crate::editing::history::{EditorHistory, HistoryEntry};
use bevy::prelude::*;
use egui::{RichText, ScrollArea, Ui};

pub fn history_tab(In(mut ui): In<Ui>, history: Res<EditorHistory>) {
    let position = history.position();
    let saved = history.saved_position();
    let entries: Vec<_> = history.entries().collect();

    if entries.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.weak(t!("tab.history.empty"));
        });
        return;
    }

    ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(&mut ui, |ui| {
            for (index, entry) in entries.iter().enumerate() {
                if saved == Some(index) {
                    saved_marker(ui);
                }
                entry_row(ui, entry, index + 1 == position);
            }
            if saved == Some(entries.len()) {
                saved_marker(ui);
            }
        });
}

fn entry_row(ui: &mut Ui, entry: &HistoryEntry, current: bool) {
    let time = chrono::DateTime::<chrono::Local>::from(entry.edited_at);
    ui.horizontal(|ui| {
        // Reserve the marker width on non-current rows to keep entries aligned.
        let marker = if current {
            RichText::new(egui_phosphor::regular::CARET_RIGHT)
        } else {
            RichText::new(egui_phosphor::regular::CARET_RIGHT).color(egui::Color32::TRANSPARENT)
        };
        ui.label(marker);
        ui.label(RichText::new(time.format("%H:%M:%S").to_string()).monospace().weak());
        let description = if entry.applied {
            RichText::new(entry.description)
        } else {
            RichText::new(entry.description).weak()
        };
        ui.add(egui::Label::new(description).truncate());
    });
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
