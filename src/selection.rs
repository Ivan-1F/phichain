use crate::chart::event::LineEvent;
use crate::chart::note::Note;
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
    note_query: Query<&Note>,
    event_query: Query<&LineEvent>,

    keyboard: Res<ButtonInput<KeyCode>>,

    selected_notes_query: Query<Entity, (With<Selected>, With<Note>, Without<LineEvent>)>,
    selected_events_query: Query<Entity, (With<Selected>, With<LineEvent>, Without<Note>)>,

    selected_notes_and_events_query: Query<
        Entity,
        (With<Selected>, Or<(With<Note>, With<LineEvent>)>),
    >,
) {
    for event in select_events.read() {
        if keyboard.pressed(KeyCode::ControlLeft) {
            // TODO: on macOS this is SuperLeft
            // selecting both notes and events is not allowed
            if note_query.get(event.0).is_ok() {
                // target is note, unselect all events
                for entity in &selected_events_query {
                    commands.entity(entity).remove::<Selected>();
                }
            }
            if event_query.get(event.0).is_ok() {
                // target is event, unselect all notes
                for entity in &selected_notes_query {
                    commands.entity(entity).remove::<Selected>();
                }
            }
        } else {
            // unselect all notes and events
            for entity in &selected_notes_and_events_query {
                commands.entity(entity).remove::<Selected>();
            }
        }

        commands.entity(event.0).insert(Selected);
    }
}
