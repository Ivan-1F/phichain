pub mod nonblocking;

use crate::curve_note_track::CurveNoteTrack;
use crate::event::EventOf;
use crate::illustration::{load_illustration, open_illustration};
use anyhow::Context;
use bevy::prelude::*;
use phichain_chart::project::Project;
use phichain_chart::serialization::{PhichainChart, SerializedLine};
use std::collections::HashMap;

/// Load a project to the world using a [`Commands`]
///
/// # Resources and entities involved when loading projects
///
/// - A entity with component [Illustration] will be spawned into the world, [IllustrationAssetId] will be inserted into the world
///
/// ---
///
/// - [phichain_chart::offset::Offset] will be inserted into the world
/// - [phichain_chart::bpm_list::BpmList] will be inserted into the world
/// - Entities with components [`Line`] and [`Note`] will be spawned into the world, with parent-child relationship
pub fn load_project(project: &Project, commands: &mut Commands) -> anyhow::Result<()> {
    let json = std::fs::read_to_string(project.path.chart_path())?;
    let chart = PhichainChart::from_json_str(&json).context("Failed to parse chart")?;
    load(chart, commands);

    if let Some(illustration_path) = project.path.illustration_path() {
        // TODO: handle error
        if let Ok(illustration) = open_illustration(illustration_path) {
            load_illustration(illustration, commands);
        }
    }

    Ok(())
}

fn load_line(line: SerializedLine, commands: &mut Commands, parent: Option<Entity>) -> Entity {
    let id = commands
        .spawn((line.line, line.id))
        .with_children(|parent| {
            let mut note_entities = HashMap::new();

            for note in line.notes {
                let entity = parent.spawn((note.data, note.id)).id();
                note_entities.insert(note.id, entity);
            }

            for track in line.curve_note_tracks {
                if let (Some(from), Some(to)) =
                    (note_entities.get(&track.from), note_entities.get(&track.to))
                {
                    parent.spawn((
                        CurveNoteTrack {
                            from: Some(*from),
                            to: Some(*to),
                            options: track.options,
                        },
                        track.id,
                    ));
                } else {
                    warn!("dangling curve note track detected: {:?}", track);
                }
            }
        })
        .id();

    for event in line.events {
        commands.spawn((event.data, event.id, EventOf(id)));
    }

    if let Some(parent) = parent {
        commands.entity(id).insert(ChildOf(parent));
    }

    for child in line.children {
        load_line(child, commands, Some(id));
    }

    id
}

/// Load a chart to the world using a [`Commands`]
fn load(chart: PhichainChart, commands: &mut Commands) {
    commands.insert_resource(chart.offset);
    commands.insert_resource(chart.bpm_list);

    for line in chart.lines {
        load_line(line, commands, None);
    }
}
