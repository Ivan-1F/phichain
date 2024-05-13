use serde::{Deserialize, Serialize};

use crate::{
    chart::{event::LineEvent, note::Note},
    timing::BpmList,
};

#[derive(Serialize, Deserialize)]
pub struct PhiChainChart {
    pub bpm_list: BpmList,
    pub lines: Vec<LineWrapper>,
}

impl PhiChainChart {
    pub fn new(bpm_list: BpmList, lines: Vec<LineWrapper>) -> Self {
        Self {
            bpm_list,
            lines,
        }
    }
}

/// A wrapper struct to handle line serialzation and deserialzation
#[derive(Serialize, Deserialize)]
pub struct LineWrapper(pub Vec<Note>, pub Vec<LineEvent>);
