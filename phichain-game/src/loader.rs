use crate::curve_note_track::CurveNoteTrack;
use crate::illustration::load_illustration;
use anyhow::Context;
use bevy::prelude::*;
use phichain_chart::event::LineEventBundle;
use phichain_chart::line::LineBundle;
use phichain_chart::migration::migrate;
use phichain_chart::note::NoteBundle;
use phichain_chart::project::Project;
use phichain_chart::serialization::{PhichainChart, SerializedLine};
use serde_json::Value;
use std::fs::File;

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
/// - Entities with components [`LineBundle`] and [`NoteBundle`] will be spawned into the world, with parent-child relationship
pub fn load_project(project: &Project, commands: &mut Commands) -> anyhow::Result<()> {
    let file = File::open(project.path.chart_path())?;
    load(file, commands)?;

    if let Some(illustration_path) = project.path.illustration_path() {
        load_illustration(illustration_path, commands);
    }

    Ok(())
}

fn load_line(line: SerializedLine, commands: &mut Commands, parent: Option<Entity>) -> Entity {
    let id = commands
        .spawn(LineBundle::new(line.line))
        .with_children(|parent| {
            let mut note_entity_order = vec![];

            for note in line.notes {
                let id = parent.spawn(NoteBundle::new(note)).id();
                note_entity_order.push(id);
            }
            for event in line.events {
                parent.spawn(LineEventBundle::new(event));
            }
            for track in line.curve_note_tracks {
                if let (Some(from), Some(to)) = (
                    note_entity_order.get(track.from),
                    note_entity_order.get(track.to),
                ) {
                    parent.spawn(CurveNoteTrack {
                        from: Some(*from),
                        to: Some(*to),
                        options: track.options,
                    });
                } else {
                    warn!("invalid curve note track detected: {:?}", track);
                }
            }
        })
        .id();

    if let Some(parent) = parent {
        commands.entity(id).set_parent(parent);
    }

    for child in line.children {
        load_line(child, commands, Some(id));
    }

    id
}

/// Load a chart to the world using a [`Commands`]
fn load(file: File, commands: &mut Commands) -> anyhow::Result<()> {
    let chart: Value = serde_json::from_reader(file).context("Failed to load chart")?;
    let migrated = migrate(&chart).context("Migration failed")?;
    let chart: PhichainChart =
        serde_json::from_value(migrated).context("Failed to deserialize chart")?;

    commands.insert_resource(chart.offset);
    commands.insert_resource(chart.bpm_list);

    let mut first_line_id: Option<Entity> = None;
    for line in chart.lines {
        let id = load_line(line, commands, None);
        if first_line_id.is_none() {
            first_line_id = Some(id)
        }
    }

    Ok(())
}
