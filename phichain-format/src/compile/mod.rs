use phichain_chart::beat::Beat;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
use phichain_chart::serialization::SerializedLine;

pub mod steps;

/// Encode the implicit starting values as events for external chart formats.
pub(crate) fn events_for_export(line: &SerializedLine) -> impl Iterator<Item = LineEvent> + '_ {
    let initial = LineEventKind::ALL.into_iter().filter_map(move |kind| {
        let end_beat = line
            .events
            .iter()
            .filter(|event| event.data.kind == kind)
            .map(|event| event.data.start_beat)
            .min()
            .unwrap_or(Beat::ONE);

        (end_beat > Beat::ZERO).then(|| LineEvent {
            kind,
            start_beat: Beat::ZERO,
            end_beat,
            value: LineEventValue::constant(kind.default_value()),
        })
    });

    initial.chain(line.events.iter().map(|event| event.data))
}
