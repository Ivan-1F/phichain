use crate::utils::{find_ranges, EventSequence};
use phichain_chart::event::LineEventValue;
use phichain_chart::serialization::{PhichainChart, SerializedLine};

fn is_unit_line(line: &SerializedLine) -> bool {
    // if the line has any notes, it is not a unit line
    if line.notes.len() > 0 {
        return false;
    }

    // if the line does not have any events, it is a unit line
    if line.events.len() == 0 {
        return true;
    }

    // all the opacity events are with values less than 0.0, meaning the line is invisible at all time
    if line.events.opacity().iter().all(|x| match x.value {
        LineEventValue::Transition { start, end, .. } => start <= 0.0 && end <= 0.0,
        LineEventValue::Constant(value) => value <= 0.0,
    }) {
        return true;
    }

    let visible_ranges = find_ranges(&line.events, |state| state.is_visible());
    tracing::debug!(
        "Visible ranges for `{}`: {:?}",
        line.line.name,
        visible_ranges
    );

    visible_ranges.is_empty()
}

/// Removes all unit lines from the chart
///
/// Unit line: a line that does not have any notes on it and is invisible at any given time
pub fn remove_unit_lines(chart: PhichainChart) -> PhichainChart {
    let lines: Vec<_> = chart
        .lines
        .clone()
        .into_iter()
        .filter(|x| !is_unit_line(x))
        .collect();

    tracing::info!("Removed {} unit lines", chart.lines.len() - lines.len());

    PhichainChart { lines, ..chart }
}
