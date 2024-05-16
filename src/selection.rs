use bevy::prelude::*;

use crate::project::project_loaded;

#[derive(Resource)]
pub struct SelectedLine(pub Entity);

#[derive(Component, Debug)]
pub struct Selected;

/// Select a [Entity] in the world
#[derive(Event)]
pub struct SelectEvent(pub Entity);

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SelectEvent>()
            .add_systems(Update, handle_select_event.run_if(project_loaded()));
    }
}

pub fn handle_select_event(
    mut commands: Commands,
    mut select_events: EventReader<SelectEvent>,
) {
    for event in select_events.read() {
        commands.entity(event.0).insert(Selected);
    }
}
