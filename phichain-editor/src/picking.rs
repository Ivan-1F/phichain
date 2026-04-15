use crate::selection::{Select, SelectedLine};
use bevy::prelude::*;
use phichain_chart::line::{Line, LineOpacity};
use phichain_chart::note::Note;

pub struct PickingPlugin;

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_click_note);
        app.add_observer(on_click_line);
    }
}

fn on_click_note(
    mut click: On<Pointer<Click>>,
    note_query: Query<(), With<Note>>,
    mut select: MessageWriter<Select>,
) {
    if click.button != PointerButton::Primary {
        return;
    }
    if note_query.contains(click.entity) {
        select.write(Select(vec![click.entity]));
        click.propagate(false); // Don't bubble to parent Line
    }
}

fn on_click_line(
    click: On<Pointer<Click>>,
    line_query: Query<&LineOpacity, With<Line>>,
    selected_line: Option<ResMut<SelectedLine>>,
) {
    if click.button != PointerButton::Primary {
        return;
    }
    if let Some(mut selected_line) = selected_line {
        if let Ok(opacity) = line_query.get(click.entity) {
            if opacity.0 > 0.0 {
                selected_line.0 = click.entity;
            }
        }
    }
}
