use crate::editing::history::Edits;
use crate::selection::SelectedLine;
use bevy::prelude::*;
use egui::Ui;
use phichain_chart::line::Line;

pub fn line_inspector(
    In(mut ui): In<Ui>,
    line_query: Query<&Line>,
    selected_line: Res<SelectedLine>,
    mut edits: Edits,
) -> Result {
    let line = line_query.get(selected_line.0)?;
    ui.label(t!("tab.inspector.line.title"));
    ui.separator();
    edits
        .component(selected_line.0, line, t!("history.edit_line"))
        .field(
            &mut ui,
            "name",
            t!("tab.inspector.line.name"),
            |ui, line| {
                ui.add(egui::TextEdit::singleline(&mut line.name));
            },
        );
    Ok(())
}
