use crate::note::Note;
use crate::primitive::event::LineEvent;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub notes: Vec<Note>,
    pub events: Vec<LineEvent>,
}
