use crate::events::{EditorEvent, EditorEventAppExt};
use crate::selection::SelectedLine;
use bevy::prelude::*;
use bon::Builder;
use phichain_chart::event::LineEventBundle;
use phichain_chart::line::{Line, LineBundle};
use phichain_chart::note::NoteBundle;
use phichain_chart::serialization::LineWrapper;
use phichain_game::GameSet;

pub struct LineEventPlugin;

impl Plugin for LineEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DespawnLineEvent>()
            .add_systems(Update, handle_despawn_line_event_system.in_set(GameSet))
            .add_editor_event::<SpawnLineEvent>();
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

#[derive(Debug, Clone, Event, Builder)]
pub struct SpawnLineEvent {
    /// The line data
    line: LineWrapper,
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
