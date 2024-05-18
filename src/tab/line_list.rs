use crate::chart::event::LineEvent;
use crate::chart::line::Line;
use crate::chart::note::Note;
use crate::selection::SelectedLine;
use bevy::prelude::*;
use egui::Ui;

pub fn line_list_tab(
    In(ui): In<&mut Ui>,
    line_query: Query<(&Children, Entity), With<Line>>,
    note_query: Query<&Note>,
    event_query: Query<&LineEvent>,

    mut selected_line: ResMut<SelectedLine>,
) {
    for (index, (line, entity)) in line_query.iter().enumerate() {
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
        if ui
            .selectable_label(
                selected_line.0 == entity,
                format!("Line #{}: {} notes, {} events", index, notes, events),
            )
            .clicked()
        {
            selected_line.0 = entity;
        }
    }
}
