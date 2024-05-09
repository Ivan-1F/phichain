use bevy::prelude::*;

#[derive(Resource)]
pub struct SelectedLine(pub Entity);

#[derive(Component, Debug)]
pub struct Selected;

#[derive(Event)]
pub struct SelectNoteEvent(pub Entity);

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SelectNoteEvent>()
            .add_systems(Update, handle_select_event);
    }
}

pub fn handle_select_event(
    mut commands: Commands,
    mut select_events: EventReader<SelectNoteEvent>,
) {
    for event in select_events.read() {
        commands.entity(event.0).insert(Selected);
    }
}
