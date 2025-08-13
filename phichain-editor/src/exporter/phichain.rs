use anyhow::Context;
use bevy::prelude::*;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::line::{Line, LineTimestamp};
use phichain_chart::offset::Offset;

use super::Exporter;
use crate::serialization::SerializeLine;
use phichain_chart::serialization::{PhichainChart, SerializedLine};

pub struct PhichainExporter;

impl Exporter for PhichainExporter {
    fn export(world: &mut World) -> anyhow::Result<String> {
        let bpm_list = world.resource::<BpmList>().clone();
        let offset = world.resource::<Offset>().0;
        let mut chart = PhichainChart::new(offset, bpm_list, vec![]);

        let mut line_query =
            world.query_filtered::<(Entity, &LineTimestamp), (With<Line>, Without<ChildOf>)>();

        let lines = line_query
            .iter(world)
            .sort::<&LineTimestamp>()
            .collect::<Vec<_>>();

        for (entity, _) in lines {
            chart
                .lines
                .push(SerializedLine::serialize_line(world, entity));
        }

        serde_json::to_string(&chart).context("Failed to export chart as phichain")
    }
}
