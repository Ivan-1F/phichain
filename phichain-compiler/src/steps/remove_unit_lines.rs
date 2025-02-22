use crate::lifetime::find_lifetime;
use phichain_chart::serialization::PhichainChart;

/// Removes all unit lines from the chart
///
/// Unit line: a line that does not have any notes on it and is invisible at any given time
pub fn remove_unit_lines(chart: PhichainChart) -> PhichainChart {
    let lines: Vec<_> = chart
        .lines
        .clone()
        .into_iter()
        .filter(|x| !find_lifetime(x).is_unit())
        .collect();

    tracing::info!("Removed {} unit lines", chart.lines.len() - lines.len());

    PhichainChart { lines, ..chart }
}
