use crate::selection::SelectedLine;
use crate::ui::latch;
use bevy::prelude::*;
use egui::Ui;
use phichain_chart::line::Line;

pub fn line_inspector(
    In(mut ui): In<Ui>,
    mut line_query: Query<&mut Line>,
    selected_line: Res<SelectedLine>,
) -> Result {
    let mut line = line_query.get_mut(selected_line.0)?;

    egui::Grid::new("inspector_grid")
        .num_columns(2)
        .spacing([20.0, 2.0])
        .striped(true)
        .show(&mut ui, |ui| {
            let result = latch::latch(ui, "line", line.clone(), |ui| {
                let mut finished = false;

                ui.label(t!("tab.inspector.line.name"));
                let response = ui.text_edit_singleline(&mut line.name);
                finished |= response.lost_focus();
                ui.end_row();

                finished
            });

            if let Some(from) = result {
                if from != line.clone() {
                    // TODO: write to history to support undo/redo
                }
            }
        });

    Ok(())
}
