use crate::bpm_list::BpmList;
use crate::format::Format;
use crate::primitive::line::Line;
use serde::{Deserialize, Serialize};

pub mod event;
pub mod line;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveChart {
    pub format: u64,
    pub offset: f32,
    pub bpm_list: BpmList,
    pub lines: Vec<Line>,
}

impl Default for PrimitiveChart {
    fn default() -> Self {
        Self {
            format: 1,
            offset: Default::default(),
            bpm_list: Default::default(),
            lines: Default::default(),
        }
    }
}

impl Format for PrimitiveChart {
    fn into_primitive(self) -> anyhow::Result<PrimitiveChart> {
        Ok(self)
    }

    fn from_primitive(phichain: PrimitiveChart) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(phichain)
    }
}
