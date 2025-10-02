use phichain_chart::curve_note_track::generate_notes;
use phichain_chart::serialization::PhichainChart;

/// Evaluate all curve note tracks and transform them into real notes
pub fn evaluate_curve_note_tracks(chart: PhichainChart) -> PhichainChart {
    // This step is after `merge_children_line`, so all the lines should have zero children
    let lines: Vec<_> = chart
        .lines
        .iter()
        .map(|line| {
            let mut line = line.clone();
            for track in &line.curve_note_tracks {
                if let (Some(from), Some(to)) =
                    (line.notes.get(track.from), line.notes.get(track.to))
                {
                    let mut notes = generate_notes(*from, *to, &track.options);
                    line.notes.append(&mut notes);
                }
            }

            line.curve_note_tracks.clear();

            line
        })
        .collect();

    PhichainChart { lines, ..chart }
}
