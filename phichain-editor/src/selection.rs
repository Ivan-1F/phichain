use crate::editing::pending::Pending;
use bevy::prelude::*;
use phichain_chart::event::LineEvent;
use phichain_chart::note::Note;

use crate::project::project_loaded;
use crate::utils::compat::ControlKeyExt;

#[derive(Resource)]
pub struct SelectedLine(pub Entity);

#[derive(Component, Debug)]
pub struct Selected;

#[derive(Component, Debug)]
pub struct CanNotBeSelected;

/// Select a vec of [Entity] in the world
#[derive(Event)]
pub struct SelectEvent(pub Vec<Entity>);

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

    keyboard: Res<ButtonInput<KeyCode>>,

    can_not_be_selected_query: Query<&CanNotBeSelected>,
    pending_query: Query<&Pending>,

    selected_notes_and_events_query: Query<
        Entity,
        (With<Selected>, Or<(With<Note>, With<LineEvent>)>),
    >,
) {
    for event in select_events.read() {
        if !keyboard.pressed(KeyCode::control()) {
            // unselect all notes and events
            for entity in &selected_notes_and_events_query {
                commands.entity(entity).remove::<Selected>();
            }
        }

        for entity in &event.0 {
            if can_not_be_selected_query.get(*entity).is_ok() {
                continue;
            }
            // pending entities cannot be selected
            if pending_query.get(*entity).is_ok() {
                continue;
            }
            commands.entity(*entity).insert(Selected);
        }
    }
}
