use crate::action::ActionRegistrationExt;
use crate::editing::command::line::CreateLine;
use crate::editing::command::EditorCommand;
use crate::editing::DoCommandEvent;
use crate::hotkey::modifier::Modifier;
use crate::hotkey::Hotkey;
use bevy::app::App;
use bevy::prelude::*;

pub struct LineEditingPlugin;

impl Plugin for LineEditingPlugin {
    fn build(&self, app: &mut App) {
        app.add_heavy_action(
            "phichain.create_line",
            create_line_system,
            Some(Hotkey::new(KeyCode::KeyN, vec![Modifier::Control])),
        );
    }
}

fn create_line_system(mut do_command_event: EventWriter<DoCommandEvent>) -> Result {
    do_command_event.write(DoCommandEvent(EditorCommand::CreateLine(CreateLine::new())));
    // TODO: switch to this line

    Ok(())
}
