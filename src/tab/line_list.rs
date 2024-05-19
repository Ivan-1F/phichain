use crate::chart::event::LineEvent;
use crate::chart::line::{Line, LineOpacity, LinePosition, LineRotation};
use crate::chart::note::Note;
use crate::selection::SelectedLine;
use bevy::prelude::*;
use egui::Ui;

pub fn line_list_tab(
    In(ui): In<&mut Ui>,
    line_query: Query<
        (
            &Children,
            Entity,
            &LinePosition,
            &LineRotation,
            &LineOpacity,
        ),
        With<Line>,
    >,
    note_query: Query<&Note>,
    event_query: Query<&LineEvent>,

    mut selected_line: ResMut<SelectedLine>,
) {
    for (index, (line, entity, position, rotation, opacity)) in line_query.iter().enumerate() {
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
                format!(
                    "Line #{}\n\
                     {} notes, {} events\n\
                     Pos: ({:.2}, {:.2})\n\
                     Rot: {:.2}\n\
                     Opa: {:.2}",
                    index, notes, events, position.0.x, position.0.y, rotation.0, opacity.0
                ),
            )
            .clicked()
        {
            selected_line.0 = entity;
        }
    }
}
