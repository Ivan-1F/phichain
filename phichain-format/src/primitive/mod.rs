use crate::primitive::line::Line;
use phichain_chart::bpm_list::BpmList;
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
