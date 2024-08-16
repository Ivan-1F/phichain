use crate::primitive::event::LineEvent;
use crate::primitive::note::Note;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub notes: Vec<Note>,
    pub events: Vec<LineEvent>,
}
