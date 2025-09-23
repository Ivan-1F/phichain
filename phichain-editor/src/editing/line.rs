use crate::action::ActionRegistrationExt;
use crate::editing::command::line::{CreateLine, CreateLineFromSelected};
use crate::editing::command::EditorCommand;
use crate::editing::DoCommandEvent;
use crate::hotkey::modifier::Modifier;
use crate::hotkey::Hotkey;
use crate::selection::SelectedLine;
use bevy::app::App;
use bevy::prelude::*;

pub struct LineEditingPlugin;

impl Plugin for LineEditingPlugin {
    fn build(&self, app: &mut App) {
        app.add_heavy_action(
            "phichain.create_line",
            create_line_system,
            Some(Hotkey::new(KeyCode::KeyN, vec![Modifier::Control])),
        )
        .add_heavy_action(
            "phichain.create_line_from_selected",
            create_line_from_selected_system,
            Some(Hotkey::new(
                KeyCode::KeyN,
                vec![Modifier::Control, Modifier::Shift],
            )),
        );
    }
}

fn create_line_system(mut do_command_event: EventWriter<DoCommandEvent>) -> Result {
    do_command_event.write(DoCommandEvent(EditorCommand::CreateLine(CreateLine::new())));
    // TODO: switch to this line

    Ok(())
}

fn create_line_from_selected_system(
    mut do_command_event: EventWriter<DoCommandEvent>,
    selected_line: Res<SelectedLine>,
) -> Result {
    do_command_event.write(DoCommandEvent(EditorCommand::CreateLineFromSelected(
        CreateLineFromSelected::new(selected_line.0),
    )));
    // TODO: switch to this line

    Ok(())
}
