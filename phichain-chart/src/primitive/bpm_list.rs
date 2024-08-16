use crate::beat::Beat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct BpmPoint {
    pub beat: Beat,
    pub bpm: f32,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BpmList(pub Vec<BpmPoint>);
