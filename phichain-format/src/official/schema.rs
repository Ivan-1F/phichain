//! Phigros official json chart format

use crate::primitive::PrimitiveChart;
use crate::Format;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr, Debug)]
#[repr(u8)]
pub enum OfficialNoteKind {
    Tap = 1,
    Drag = 2,
    Hold = 3,
    Flick = 4,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OfficialNote {
    #[serde(rename = "type")]
    pub kind: OfficialNoteKind,
    pub time: f32,
    #[serde(rename = "holdTime")]
    pub hold_time: f32,
    #[serde(rename = "positionX")]
    pub x: f32,
    pub speed: f32,

    #[serde(rename = "floorPosition")]
    pub floor_position: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OfficialNumericLineEvent {
    #[serde(rename = "startTime")]
    pub start_time: f32,
    #[serde(rename = "endTime")]
    pub end_time: f32,
    pub start: f32,
    pub end: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OfficialPositionLineEvent {
    #[serde(rename = "startTime")]
    pub start_time: f32,
    #[serde(rename = "endTime")]
    pub end_time: f32,
    #[serde(rename = "start")]
    pub start_x: f32,
    // formatVersion 1 does not have start2
    #[serde(rename = "start2", default)]
    pub start_y: f32,
    #[serde(rename = "end")]
    pub end_x: f32,
    // formatVersion 1 does not have end2
    #[serde(rename = "end2", default)]
    pub end_y: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OfficialSpeedEvent {
    #[serde(rename = "startTime")]
    pub start_time: f32,
    #[serde(rename = "endTime")]
    pub end_time: f32,
    pub value: f32,

    #[serde(rename = "floorPosition", default)]
    pub floor_position: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OfficialLine {
    pub bpm: f32,

    #[serde(rename = "judgeLineMoveEvents")]
    pub move_events: Vec<OfficialPositionLineEvent>,
    #[serde(rename = "judgeLineRotateEvents")]
    pub rotate_events: Vec<OfficialNumericLineEvent>,
    #[serde(rename = "judgeLineDisappearEvents")]
    pub opacity_events: Vec<OfficialNumericLineEvent>,
    #[serde(rename = "speedEvents")]
    pub speed_events: Vec<OfficialSpeedEvent>,

    #[serde(rename = "notesAbove")]
    pub notes_above: Vec<OfficialNote>,
    #[serde(rename = "notesBelow")]
    pub notes_below: Vec<OfficialNote>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OfficialChart {
    #[serde(rename = "formatVersion")]
    pub format_version: u32,
    pub offset: f32,
    #[serde(rename = "judgeLineList")]
    pub lines: Vec<OfficialLine>,
}

impl Format for OfficialChart {
    fn into_primitive(self) -> anyhow::Result<PrimitiveChart> {
        unimplemented!("use official_to_phichain instead")
    }

    fn from_primitive(_: PrimitiveChart) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        unimplemented!("use phichain_to_official instead")
    }
}
