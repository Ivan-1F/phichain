use crate::action::ActionRegistrationExt;
use crate::editing::pending::Pending;
use crate::hotkey::modifier::Modifier;
use crate::hotkey::Hotkey;
use crate::project::project_loaded;
use crate::utils::compat::ControlKeyExt;
use bevy::prelude::*;
use phichain_game::curve_note_track::CurveNote;
use phichain_game::utils::query_ordered_lines;

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

        for i in 0..10 {
            app.add_action(
                format!("phichain.select_line_{}", i).as_str(),
                move |world: &mut World| {
                    if let Some(entity) =
                        query_ordered_lines(world).get(if i == 0 { 10 } else { i } - 1)
                    {
                        world.resource_mut::<SelectedLine>().0 = *entity;
                    }
                },
                Some(Hotkey::new(
                    match i {
                        0 => KeyCode::Digit0,
                        1 => KeyCode::Digit1,
                        2 => KeyCode::Digit2,
                        3 => KeyCode::Digit3,
                        4 => KeyCode::Digit4,
                        5 => KeyCode::Digit5,
                        6 => KeyCode::Digit6,
                        7 => KeyCode::Digit7,
                        8 => KeyCode::Digit8,
                        9 => KeyCode::Digit9,
                        _ => unreachable!(),
                    },
                    vec![Modifier::Control],
                )),
            );
        }
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
