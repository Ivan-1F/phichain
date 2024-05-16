use crate::chart::event::LineEvent;
use crate::chart::line::Line;
use crate::chart::note::Note;
use bevy::prelude::*;
use egui::Ui;

// use crate::translation::Translator;

pub fn line_list_tab(
    In(ui): In<&mut Ui>,
    line_query: Query<&Children, With<Line>>,
    note_query: Query<&Note>,
    event_query: Query<&LineEvent>,
    // translator: Translator,
) {
    for (index, line) in line_query.iter().enumerate() {
        let notes = line
            .iter()
            .filter(|child| note_query.get(**child).is_ok())
            .collect::<Vec<_>>()
            .len();
        let events = line
            .iter()
            .filter(|child| event_query.get(**child).is_ok())
            .collect::<Vec<_>>()
            .len();
        ui.label(format!(
            "Line #{}: {} notes, {} events",
            index, notes, events
        ));
    }
}
