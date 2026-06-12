use phichain_chart::bpm_list::BpmList;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
use phichain_chart::line::Line;
use phichain_chart::note::{Note, NoteKind};
use phichain_chart::serialization::{PhichainChart, SerializedLine};

/// Generate a synthetic chart.json for load benchmarking
///
/// Usage: gen_chart <output-path> <target-size-mb>
fn main() {
    let mut args = std::env::args().skip(1);
    let out = args.next().expect("usage: gen_chart <output-path> <target-size-mb>");
    let target_mb: usize = args
        .next()
        .expect("usage: gen_chart <output-path> <target-size-mb>")
        .parse()
        .unwrap();

    let per_line_notes = 4000;
    let per_line_events = 4000;
    let mut lines = vec![];
    let mut total = 0usize;
    while total < target_mb * 1024 * 1024 {
        let notes: Vec<Note> = (0..per_line_notes)
            .map(|i| {
                Note::new(
                    NoteKind::Tap,
                    true,
                    phichain_chart::beat!(i, 1, 3),
                    (i % 1000) as f32 - 500.0,
                    1.0,
                )
            })
            .collect();
        let events: Vec<LineEvent> = (0..per_line_events)
            .map(|i| LineEvent {
                kind: LineEventKind::X,
                start_beat: phichain_chart::beat!(i, 1, 3),
                end_beat: phichain_chart::beat!(i + 1, 1, 3),
                value: LineEventValue::transition(
                    0.25,
                    0.75,
                    phichain_chart::easing::Easing::Linear,
                ),
            })
            .collect();
        lines.push(SerializedLine::new(Line::default(), notes, events, vec![], vec![]));
        total += (per_line_notes * 110 + per_line_events * 160) as usize;
    }

    let chart = PhichainChart::new(0.0, BpmList::default(), lines);
    let json = serde_json::to_string(&chart).unwrap();
    println!("chart json size: {:.1} MB", json.len() as f64 / 1e6);
    std::fs::write(out, json).unwrap();
}
