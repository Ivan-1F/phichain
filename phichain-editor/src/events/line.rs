use crate::events::{EditorEvent, EditorEventAppExt};
use crate::selection::SelectedLine;
use crate::utils::entity::replace_with_empty;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bon::Builder;
use phichain_chart::event::LineEventBundle;
use phichain_chart::line::{Line, LineBundle};
use phichain_chart::note::NoteBundle;
use phichain_chart::serialization::SerializedLine;

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
            Query<&Parent>,
            Query<Entity, (With<Line>, Without<Parent>)>,
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
            world.entity_mut(self.target).despawn_recursive();
        }
    }
}

#[derive(Debug, Clone, Event, Builder)]
pub struct SpawnLineEvent {
    /// The line data
    line: SerializedLine,
    /// The entity of the parent line
    parent: Option<Entity>,
    /// The target entity to spawn the line. If given, components will be inserted to this entity instead of a new entity
    target: Option<Entity>,
}

impl EditorEvent for SpawnLineEvent {
    type Output = Entity;

    // TODO: move part of the logic to phichain-game utils, duplication of phichain_game::loader::load_line()
    fn run(self, world: &mut World) -> Self::Output {
        let id = match self.target {
            None => world.spawn(LineBundle::new(self.line.line)).id(),
            Some(target) => world
                .entity_mut(target)
                .insert(LineBundle::new(self.line.line))
                .id(),
        };

        world.entity_mut(id).with_children(|parent| {
            for note in self.line.notes {
                parent.spawn(NoteBundle::new(note));
            }
            for event in self.line.events {
                parent.spawn(LineEventBundle::new(event));
            }
        });

        if let Some(parent) = self.parent {
            world.entity_mut(id).set_parent(parent);
        }

        for child in self.line.children {
            SpawnLineEvent::builder()
                .line(child)
                .parent(id)
                .build()
                .run(world);
        }

        id
    }
}
