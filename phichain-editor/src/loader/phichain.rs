use anyhow::Context;
use std::fs::File;

use bevy::prelude::*;
use phichain_chart::event::LineEventBundle;
use phichain_chart::line::LineBundle;
use phichain_chart::note::NoteBundle;
use serde_json::Value;

use crate::selection::SelectedLine;
use phichain_chart::migration::migrate;
use phichain_chart::serialization::{LineWrapper, PhiChainChart};

use super::Loader;

pub struct PhiChainLoader;

impl Loader for PhiChainLoader {
    fn load(file: File, commands: &mut Commands) -> anyhow::Result<()> {
        let chart: Value = serde_json::from_reader(file).context("Failed to load chart")?;
        let migrated = migrate(&chart).context("Migration failed")?;
        let chart: PhiChainChart =
            serde_json::from_value(migrated).context("Failed to deserialize chart")?;

        commands.insert_resource(chart.offset);
        commands.insert_resource(chart.bpm_list);

        let mut first_line_id: Option<Entity> = None;
        for LineWrapper {
            line,
            notes,
            events,
        } in chart.lines
        {
            let id = commands
                .spawn(LineBundle::new(line))
                .with_children(|parent| {
                    for note in notes {
                        parent.spawn(NoteBundle::new(note));
                    }
                    for event in events {
                        parent.spawn(LineEventBundle::new(event));
                    }
                })
                .id();

            if first_line_id.is_none() {
                first_line_id = Some(id)
            }
        }

        commands.insert_resource(SelectedLine(first_line_id.unwrap()));

        Ok(())
    }
}
