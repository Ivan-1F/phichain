use crate::selection::SelectedLine;
use bevy::prelude::*;
use phichain_chart::line::Line;
use phichain_game::GameSet;

pub struct LineEventPlugin;

impl Plugin for LineEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DespawnLineEvent>()
            .add_systems(Update, handle_despawn_line_event_system.in_set(GameSet));
    }
}

/// Despawn a line and its child lines from the world
#[derive(Debug, Clone, Event)]
pub struct DespawnLineEvent(pub Entity);

/// Despawn a line and its child lines from the world
///
/// If the target is the only root line in the world, do nothing
/// If the target is the selected line or ancestors of the selected line (despawning target causing dangling selected line), update the selected line to the first root line
fn handle_despawn_line_event_system(
    mut commands: Commands,
    mut events: EventReader<DespawnLineEvent>,
    parent_query: Query<&Parent>,
    root_line_query: Query<Entity, (With<Line>, Without<Parent>)>,
    mut selected_line: ResMut<SelectedLine>,
) {
    for DespawnLineEvent(entity) in events.read() {
        debug!("attempt to despawn line {:?}", entity);
        let is_root = parent_query.get(*entity).is_err();
        let other_root_lines = root_line_query
            .iter()
            .filter(|x| x != entity)
            .collect::<Vec<_>>();
        if is_root && other_root_lines.is_empty() {
            // attempt to remove the only root line -> do nothing
            debug!(
                "attempt to despawn the only root line {:?}, skipping",
                entity
            );
            continue;
        };

        // despawning target causing dangling selected line -> changing the selected line to the first root line
        if selected_line.0 == *entity
            || parent_query
                .iter_ancestors(selected_line.0)
                .any(|x| x == *entity)
        {
            // unwrap: other_root_lines.len() != 0
            selected_line.0 = *other_root_lines.first().unwrap();
        }

        debug!("despawned line {:?}", entity);
        commands.entity(*entity).despawn_recursive();
    }
}
