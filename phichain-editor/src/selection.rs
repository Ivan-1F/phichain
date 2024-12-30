use crate::editing::pending::Pending;
use crate::project::project_loaded;
use crate::utils::compat::ControlKeyExt;
use bevy::prelude::*;
use phichain_game::curve_note_track::CurveNote;

#[derive(Resource)]
pub struct SelectedLine(pub Entity);

#[derive(Component, Debug)]
pub struct Selected;

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

    curve_note_query: Query<&CurveNote>,
    pending_query: Query<&Pending>,

    selected_query: Query<Entity, With<Selected>>,
) {
    for event in select_events.read() {
        if !keyboard.pressed(KeyCode::control()) {
            // unselect everything
            for entity in &selected_query {
                commands.entity(entity).remove::<Selected>();
            }
        }

        for entity in &event.0 {
            if let Ok(curve_note) = curve_note_query.get(*entity) {
                commands.entity(curve_note.0).insert(Selected);
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
