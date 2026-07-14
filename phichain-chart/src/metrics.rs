use crate::serialization::SerializedLine;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct ChartMetrics {
    pub lines: usize,
    pub notes: usize,
    pub events: usize,
}

impl ChartMetrics {
    pub fn collect(lines: &[SerializedLine]) -> Self {
        let mut metrics = Self::default();
        for line in lines {
            metrics.lines += 1;
            metrics.notes += line.notes.len();
            metrics.events += line.events.len();
            let child = Self::collect(&line.children);
            metrics.lines += child.lines;
            metrics.notes += child.notes;
            metrics.events += child.events;
        }
        metrics
    }
}
