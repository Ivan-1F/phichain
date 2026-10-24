pub mod text_utils;

use crate::line::LineOrder;
use bevy::prelude::{ChildOf, Children, Entity, With, Without, World};
use phichain_chart::line::Line;

/// Get all lines flattened with the order
pub fn query_ordered_lines(world: &mut World) -> Vec<Entity> {
    let mut query = world.query_filtered::<(Entity, &LineOrder), (Without<ChildOf>, With<Line>)>();
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
    ordered_lines.push(entity);
    let mut query = world.query::<(&LineOrder, Option<&Children>)>();
    let children = query
        .get(world, entity)
        .ok()
        .and_then(|(_, children)| children);
    let mut children: Vec<_> = children
        .into_iter()
        .flatten()
        .filter_map(|child| {
            query
                .get(world, *child)
                .ok()
                .map(|(order, _)| (*child, *order))
        })
        .collect();
    children.sort_by_key(|(_, order)| *order);
    for (child, _) in children {
        add_line_and_descendants(world, child, ordered_lines);
    }
}
