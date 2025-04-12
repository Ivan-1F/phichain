pub mod text_utils;

use bevy::hierarchy::{Children, Parent};
use bevy::prelude::{Entity, With, Without, World};
use phichain_chart::line::{Line, LineTimestamp};

/// Get all lines flattened with the order
pub fn query_ordered_lines(world: &mut World) -> Vec<Entity> {
    let mut query =
        world.query_filtered::<(Entity, &LineTimestamp), (Without<Parent>, With<Line>)>();
    let mut root_entities = query.iter(world).collect::<Vec<_>>();
    root_entities.sort_by_key(|(_, timestamp)| **timestamp);
    let root_entities = root_entities
        .iter()
        .map(|(entity, _)| *entity)
        .collect::<Vec<_>>();

    let mut ordered_lines = Vec::new();
    for entity in root_entities {
        add_line_and_descendants(world, entity, &mut ordered_lines);
    }

    ordered_lines
}

fn add_line_and_descendants(world: &mut World, entity: Entity, ordered_lines: &mut Vec<Entity>) {
    let mut line_query = world.query_filtered::<&Children, With<Line>>();
    if let Ok(children) = line_query.get(world, entity) {
        ordered_lines.push(entity);

        let children_lines = children
            .iter()
            .filter(|&child| line_query.get(world, *child).is_ok())
            .copied()
            .collect::<Vec<_>>();

        for child in children_lines {
            add_line_and_descendants(world, child, ordered_lines);
        }
    }
}
