use anyhow::Context;
use bevy::prelude::*;

use crate::audio::Offset;
use crate::{
    chart::line::Line,
    serialization::{LineWrapper, PhiChainChart},
    timing::BpmList,
};

use super::Exporter;

pub struct PhiChainExporter;

impl Exporter for PhiChainExporter {
    fn export(world: &mut World) -> anyhow::Result<String> {
        let bpm_list = world.resource::<BpmList>().clone();
        let offset = world.resource::<Offset>().0;
        let mut chart = PhiChainChart::new(offset, bpm_list, vec![]);

        let mut line_query = world.query_filtered::<Entity, With<Line>>();

        let lines = line_query.iter(world).collect::<Vec<_>>();
        for entity in lines {
            chart.lines.push(LineWrapper::serialize_line(world, entity));
        }

        serde_json::to_string(&chart).context("Failed to export chart as phichain")
    }
}
