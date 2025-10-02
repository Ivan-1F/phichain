mod compile;
pub mod official;
pub mod primitive;
pub mod rpe;

use crate::primitive::PrimitiveChart;
use serde::de::DeserializeOwned;
use serde::Serialize;

// TODO
pub use compile::compile as compile_phichain_chart;
use phichain_chart::line::Line;
use phichain_chart::offset::Offset;
use phichain_chart::serialization::{PhichainChart, SerializedLine};

pub trait Format: Serialize + DeserializeOwned {
    fn into_primitive(self) -> anyhow::Result<PrimitiveChart>;

    fn from_primitive(phichain: PrimitiveChart) -> anyhow::Result<Self>
    where
        Self: Sized;
}

impl Format for PhichainChart {
    // Note: This only convert necessary types. To convert a PhichainChart to PrimitiveChart,
    // while remaining advanced features provided by phichain chart, use `phichain_format::compile_phichain_chart()` instead
    fn into_primitive(self) -> anyhow::Result<PrimitiveChart> {
        Ok(PrimitiveChart {
            offset: self.offset.0,
            bpm_list: self.bpm_list.clone(),
            lines: self
                .lines
                .iter()
                .map(|line| primitive::line::Line {
                    notes: line.notes.clone(),
                    events: line.events.iter().map(|x| (*x).into()).collect(),
                })
                .collect(),
            ..Default::default()
        })
    }

    fn from_primitive(primitive: PrimitiveChart) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            offset: Offset(primitive.offset),
            bpm_list: primitive.bpm_list,
            lines: primitive
                .lines
                .iter()
                .map(|line| {
                    SerializedLine::new(
                        Line::default(),
                        line.notes.clone(),
                        line.events.iter().map(|x| (*x).into()).collect(),
                        vec![],
                        vec![],
                    )
                })
                .collect(),
            ..Default::default()
        })
    }
}
