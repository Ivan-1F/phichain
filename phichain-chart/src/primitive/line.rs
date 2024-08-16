use serde::{Deserialize, Serialize};
use crate::event::LineEvent;
use crate::note::Note;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub notes: Vec<Note>,
    pub events: Vec<LineEvent>,
}
