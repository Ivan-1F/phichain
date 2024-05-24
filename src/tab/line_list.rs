use crate::chart::event::LineEvent;
use crate::chart::line::{Line, LineOpacity, LinePosition, LineRotation, LineSpeed};
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
            &LineSpeed,
        ),
        With<Line>,
    >,
    note_query: Query<&Note>,
    event_query: Query<&LineEvent>,

    mut selected_line: ResMut<SelectedLine>,
) {
    for (index, (line, entity, position, rotation, opacity, speed)) in line_query.iter().enumerate()
    {
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

        ui.horizontal(|ui| {
            ui.label(format!("Line #{}", index));
            let mut checked = selected_line.0 == entity;
            if ui.checkbox(&mut checked, "").clicked() {
                selected_line.0 = entity;
            }
        });

        ui.columns(2, |columns| {
            egui::Grid::new(format!("line-{}-props-grid-left", index))
                .num_columns(2)
                .show(&mut columns[0], |ui| {
                    ui.label(t!("tab.line_list.note"));
                    ui.label(format!("{}", notes));
                    ui.end_row();

                    ui.label(t!("tab.line_list.position"));
                    ui.label(format!("({:.2}, {:.2})", position.0.x, position.0.y));
                    ui.end_row();

                    ui.label(t!("tab.line_list.rotation"));
                    ui.label(format!("{:.2}", rotation.0));
                    ui.end_row();
                });
            egui::Grid::new(format!("line-{}-props-grid-right", index))
                .num_columns(2)
                .show(&mut columns[1], |ui| {
                    ui.label(t!("tab.line_list.event"));
                    ui.label(format!("{}", events));
                    ui.end_row();

                    ui.label(t!("tab.line_list.opacity"));
                    ui.label(format!("{:.2}", opacity.0));
                    ui.end_row();

                    ui.label(t!("tab.line_list.speed"));
                    ui.label(format!("{:.2}", speed.0));
                    ui.end_row();
                });
        });
        ui.separator();
    }
}
