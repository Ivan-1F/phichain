use anyhow::Context;
use bevy::prelude::*;

use crate::audio::Offset;
use crate::{
    chart::{event::LineEvent, line::Line, note::Note},
    serialzation::{LineWrapper, PhiChainChart},
    timing::BpmList,
};

use super::Exporter;

pub struct PhiChainExporter;

impl Exporter for PhiChainExporter {
    fn export(world: &mut World) -> anyhow::Result<String> {
        let bpm_list = world.resource::<BpmList>().clone();
        let offset = world.resource::<Offset>().0;
        let mut chart = PhiChainChart::new(offset, bpm_list, vec![]);

        let mut line_query = world.query_filtered::<&Children, With<Line>>();
        let mut note_query = world.query::<&Note>();
        let mut event_query = world.query::<&LineEvent>();

        for children in line_query.iter(world) {
            let mut notes: Vec<Note> = vec![];
            let mut events: Vec<LineEvent> = vec![];
            for child in children.iter() {
                if let Ok(note) = note_query.get(world, *child) {
                    notes.push(*note);
                } else if let Ok(event) = event_query.get(world, *child) {
                    events.push(*event);
                }
            }
            chart.lines.push(LineWrapper(notes, events));
        }

        serde_json::to_string(&chart).context("Failed to export chart as phichain")
    }
}
