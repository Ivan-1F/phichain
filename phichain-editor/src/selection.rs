use crate::action::ActionRegistrationExt;
use crate::hotkey::modifier::Modifier;
use crate::hotkey::Hotkey;
use crate::project::project_loaded;
use crate::utils::compat::ControlKeyExt;
use anyhow::Context;
use bevy::prelude::*;
use phichain_chart::line::Line;
use phichain_game::curve_note_track::CurveNote;
use phichain_game::utils::query_ordered_lines;
use phichain_game::GameSet;
use phichain_game::Pending;

#[derive(Resource)]
pub struct SelectedLine(pub Entity);

#[derive(Component, Debug)]
pub struct Selected;

/// Select a vec of [Entity] in the world
#[derive(Message)]
pub struct Select(pub Vec<Entity>);

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(line_removed)
            .add_message::<Select>()
            .add_systems(
                Update,
                handle_select_event.before(GameSet).run_if(project_loaded()),
            )
            .add_action(
                "phichain.unselect_all",
                unselect_all_system,
                Some(Hotkey::new(KeyCode::Escape, vec![])),
            );

        let select_line_by_offset = |offset: isize| {
            move |world: &mut World| {
                let ordered_lines = query_ordered_lines(world);
                let current_selected = world.resource::<SelectedLine>().0;

                let current_index = ordered_lines
                    .iter()
                    .position(|x| x == &current_selected)
                    .context("Failed to find index of the current selected line")?;

                let new_index = (current_index as isize).saturating_add(offset);
                if new_index >= 0 {
                    if let Some(&new_line) = ordered_lines.get(new_index as usize) {
                        world.resource_mut::<SelectedLine>().0 = new_line;
                    }
                }

                Ok(())
            }
        };

        app.add_action(
            "phichain.select_next_line",
            select_line_by_offset(1),
            Some(Hotkey::new(KeyCode::ArrowDown, vec![Modifier::Control])),
        );
        app.add_action(
            "phichain.select_prev_line",
            select_line_by_offset(-1),
            Some(Hotkey::new(KeyCode::ArrowUp, vec![Modifier::Control])),
        );

        for i in 1..10 {
            app.add_action(
                format!("phichain.select_line_{i}").as_str(),
                move |world: &mut World| {
                    if let Some(entity) = query_ordered_lines(world).get(i - 1) {
                        world.resource_mut::<SelectedLine>().0 = *entity;
                    }

                    Ok(())
                },
                Some(Hotkey::new(
                    match i {
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

fn line_removed(_: On<Remove, Line>, mut commands: Commands) {
    commands.queue(|world: &mut World| {
        let Some(selected) = world.get_resource::<SelectedLine>() else {
            return;
        };
        if world.get::<Line>(selected.0).is_some() {
            return;
        }
        if let Some(line) = query_ordered_lines(world).first().copied() {
            world.resource_mut::<SelectedLine>().0 = line;
        }
    });
}

pub fn unselect_all_system(
    mut commands: Commands,
    selected_query: Query<Entity, With<Selected>>,
) -> Result {
    for entity in &selected_query {
        commands.entity(entity).remove::<Selected>();
    }

    Ok(())
}

pub fn handle_select_event(
    mut commands: Commands,
    mut select_events: MessageReader<Select>,

    keyboard: Res<ButtonInput<KeyCode>>,

    curve_note_query: Query<&CurveNote>,
    pending_query: Query<(), With<Pending>>,

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
            if pending_query.contains(*entity) {
                continue;
            }
            commands.entity(*entity).insert(Selected);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editing::history::{open_document, EditorHistory, Edits, HistoryPlugin};
    use crate::id_index::{IdIndex, IdIndexPlugin};
    use bevy::ecs::system::RunSystemOnce;
    use phichain_chart::id::LineId;

    #[test]
    fn undoing_creation_of_the_current_line_selects_a_surviving_line() {
        let mut app = App::new();
        app.add_plugins((
            IdIndexPlugin,
            phichain_game::line::LinePlugin,
            HistoryPlugin,
        ))
        .add_observer(line_removed);
        let world = app.world_mut();
        let original = world.spawn(Line::default()).id();
        world.insert_resource(SelectedLine(original));
        open_document(world);
        let id = LineId::new();
        world
            .run_system_once(move |mut edits: Edits| {
                edits.once("create line", move |commands| {
                    commands.spawn((Line::default(), id));
                });
            })
            .unwrap();
        let created = world.resource::<IdIndex>().entity(id.uuid()).unwrap();
        world.resource_mut::<SelectedLine>().0 = created;
        world
            .resource_scope(|world, mut history: Mut<EditorHistory>| history.undo(world))
            .unwrap();
        assert_eq!(world.resource::<SelectedLine>().0, original);
        world
            .resource_scope(|world, mut history: Mut<EditorHistory>| history.redo(world))
            .unwrap();
        assert_ne!(
            world.resource::<IdIndex>().entity(id.uuid()).unwrap(),
            created
        );
        assert_eq!(world.resource::<SelectedLine>().0, original);
    }
}
