mod steps;
mod utils;

use crate::steps::{evaluate_curve_note_tracks, merge_children_line};
use phichain_chart::primitive::{Format, PrimitiveChart};
use phichain_chart::serialization::PhichainChart;

/// Compile a Phichain chart into a primitive chart
pub fn compile(chart: PhichainChart) -> anyhow::Result<PrimitiveChart> {
    let chart = merge_children_line(chart);
    let chart = evaluate_curve_note_tracks(chart);

    // TODO: move into_primitive implementation here and use compile() in into_primitive
    chart.into_primitive()
}
