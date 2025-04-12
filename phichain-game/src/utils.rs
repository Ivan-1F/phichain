use bevy::hierarchy::{Children, Parent};
use bevy::prelude::{Entity, With, Without, World};
use phichain_chart::line::{Line, LineTimestamp};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Script {
    Ascii,
    Cjk,
}

pub fn split_by_script(s: &str) -> Vec<(String, Script)> {
    if s.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut current_script = None;
    let mut current_text = String::new();

    for c in s.chars() {
        let script = if c.is_ascii() {
            Script::Ascii
        } else {
            Script::Cjk
        };

        if Some(script) != current_script {
            if !current_text.is_empty() {
                result.push((current_text.clone(), current_script.unwrap()));
                current_text.clear();
            }
            current_script = Some(script);
        }

        current_text.push(c);
    }

    if !current_text.is_empty() {
        result.push((current_text, current_script.unwrap()));
    }

    result
}

pub fn query_ordered_lines(world: &mut World) -> Vec<Entity> {
    // Get root lines sorted by timestamp
    let mut query =
        world.query_filtered::<(Entity, &LineTimestamp), (Without<Parent>, With<Line>)>();
    let mut root_entities = query.iter(world).collect::<Vec<_>>();
    root_entities.sort_by_key(|(_, timestamp)| **timestamp);
    let root_entities = root_entities
        .iter()
        .map(|(entity, _)| *entity)
        .collect::<Vec<_>>();

    // Now recursively gather all lines in order
    let mut ordered_lines = Vec::new();
    for entity in root_entities {
        add_line_and_descendants(world, entity, &mut ordered_lines);
    }

    ordered_lines
}

fn add_line_and_descendants(world: &mut World, entity: Entity, ordered_lines: &mut Vec<Entity>) {
    let mut line_query = world.query_filtered::<&Children, With<Line>>();
    if let Ok(children) = line_query.get(world, entity) {
        // Add this entity to our ordered list
        ordered_lines.push(entity);

        // Get all children that are also lines with required components
        let children_lines = children
            .iter()
            .filter(|&child| line_query.get(world, *child).is_ok())
            .copied()
            .collect::<Vec<_>>();

        // Recursively process each child line
        for child in children_lines {
            add_line_and_descendants(world, child, ordered_lines);
        }
    }
}
