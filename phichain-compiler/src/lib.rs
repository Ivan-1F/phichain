pub mod lifetime;
pub mod range;
pub mod state;
pub mod steps;
pub mod utils;

use crate::steps::{
    evaluate_curve_note_tracks, merge_children_line, remove_unit_lines, reuse_lines,
};
use phichain_chart::primitive::{Format, PrimitiveChart};
use phichain_chart::serialization::PhichainChart;

// TODO: add option to skip some steps
/// Compile a Phichain chart
#[allow(clippy::let_and_return)]
pub fn compile_only(chart: PhichainChart) -> PhichainChart {
    let chart = merge_children_line(chart);
    let chart = evaluate_curve_note_tracks(chart);
    let chart = remove_unit_lines(chart);
    let chart = reuse_lines(chart);

    chart
}

/// Compile a Phichain chart into a primitive chart
pub fn compile(chart: PhichainChart) -> anyhow::Result<PrimitiveChart> {
    let chart = compile_only(chart);

    // TODO: move into_primitive implementation here and use compile() in into_primitive
    chart.into_primitive()
}
