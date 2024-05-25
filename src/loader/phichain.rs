use std::fs::File;

use bevy::prelude::*;

use crate::{
    chart::{event::LineEventBundle, line::LineBundle, note::NoteBundle},
    selection::SelectedLine,
    serialization::{LineWrapper, PhiChainChart},
};

use super::Loader;

pub struct PhiChainLoader;

impl Loader for PhiChainLoader {
    fn load(file: File, commands: &mut Commands) {
        let chart: PhiChainChart = serde_json::from_reader(file).expect("Failed to load chart");
        commands.insert_resource(chart.offset);
        commands.insert_resource(chart.bpm_list);

        let mut first_line_id: Option<Entity> = None;
        for LineWrapper { notes, events } in chart.lines {
            let id = commands
                .spawn(LineBundle::new())
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
    }
}
