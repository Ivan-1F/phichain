use crate::selection::SelectedLine;
use bevy::prelude::*;
use egui::Ui;
use phichain_chart::line::Line;

pub fn line_inspector(
    In(mut ui): In<Ui>,
    line_query: Query<&Line>,
    selected_line: Res<SelectedLine>,
) -> Result {
    let mut line = line_query.get(selected_line.0)?.clone();

    ui.label(t!("tab.inspector.line.title"));
    ui.separator();

    egui::Grid::new("inspector_grid")
        .num_columns(2)
        .spacing([20.0, 2.0])
        .striped(true)
        .show(&mut ui, |ui| {
            ui.label(t!("tab.inspector.line.name"));
            ui.add_enabled(false, egui::TextEdit::singleline(&mut line.name));
            ui.end_row();
        });

    Ok(())
}
