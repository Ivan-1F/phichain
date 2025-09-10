use crate::events::{EditorEvent, EditorEventAppExt};
use crate::selection::SelectedLine;
use crate::utils::entity::replace_with_empty;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bon::Builder;
use phichain_chart::line::Line;
use phichain_chart::serialization::SerializedLine;
use phichain_game::line::line_bundle;

pub struct LineEventPlugin;

impl Plugin for LineEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_editor_event::<SpawnLineEvent>()
            .add_editor_event::<DespawnLineEvent>();
    }
}

/// Despawn a line and its child lines from the world
///
/// If the target is the only root line in the world, do nothing
/// If the target is the selected line or ancestors of the selected line (despawning target causing dangling selected line), update the selected line to the first root line
#[derive(Debug, Clone, Event, Builder)]
pub struct DespawnLineEvent {
    target: Entity,
    #[builder(default)] // false
    keep_entity: bool,
}

impl EditorEvent for DespawnLineEvent {
    type Output = ();

    fn run(self, world: &mut World) -> Self::Output {
        let mut state = SystemState::<(
            Query<&ChildOf>,
            Query<Entity, (With<Line>, Without<ChildOf>)>,
            ResMut<SelectedLine>,
        )>::new(world);
        let (parent_query, root_line_query, mut selected_line) = state.get_mut(world);

        debug!("attempt to despawn line {:?}", self.target);
        let is_root = parent_query.get(self.target).is_err();
        let other_root_lines = root_line_query
            .iter()
            .filter(|x| x != &self.target)
            .collect::<Vec<_>>();
        if is_root && other_root_lines.is_empty() {
            // attempt to remove the only root line -> do nothing
            debug!(
                "attempt to despawn the only root line {:?}, skipping",
                self.target
            );
            return;
        };

        // despawning target causing dangling selected line -> changing the selected line to the first root line
        if selected_line.0 == self.target
            || parent_query
                .iter_ancestors(selected_line.0)
                .any(|x| x == self.target)
        {
            // unwrap: other_root_lines.len() != 0
            selected_line.0 = *other_root_lines.first().unwrap();
        }

        debug!(
            "despawned line {:?}{}",
            self.target,
            if self.keep_entity {
                " (keep entity)"
            } else {
                ""
            }
        );
        if self.keep_entity {
            replace_with_empty(world, self.target);
        } else {
            world.entity_mut(self.target).despawn();
        }
    }
}

#[derive(Debug, Clone, Event, Builder)]
pub struct SpawnLineEvent {
    /// The line data
    line: SerializedLine,
}

impl EditorEvent for SpawnLineEvent {
    type Output = Entity;

    fn run(self, world: &mut World) -> Self::Output {
        world.spawn(line_bundle(self.line)).id()
    }
}
