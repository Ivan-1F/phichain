use crate::action::ActionRegistrationExt;
use crate::editing::history::Edits;
use crate::hotkey::modifier::Modifier;
use crate::hotkey::Hotkey;
use crate::selection::SelectedLine;
use crate::timing::ChartTime;
use bevy::prelude::*;
use num::Rational32;
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
use phichain_chart::id::Identified;
use phichain_chart::serialization::SerializedLine;
use phichain_game::event::EventIndex;
pub(crate) use phichain_game::loader::load_line as spawn_line;
use phichain_game::Pending;

pub struct LineEditingPlugin;
impl Plugin for LineEditingPlugin {
    fn build(&self, app: &mut App) {
        app.add_heavy_action(
            "phichain.create_line",
            create_line_system,
            Some(Hotkey::new(KeyCode::KeyN, vec![Modifier::Control])),
        )
        .add_heavy_action(
            "phichain.create_line_from_selected",
            create_line_from_selected_system,
            Some(Hotkey::new(
                KeyCode::KeyN,
                vec![Modifier::Control, Modifier::Shift],
            )),
        );
    }
}

/// A line as the editor creates it, opaque so it shows while the other tracks keep their defaults
///
/// The opacity event only sets the initial value, so it is kept short.
pub(crate) fn new_line() -> SerializedLine {
    SerializedLine {
        events: vec![Identified::new(LineEvent {
            kind: LineEventKind::Opacity,
            value: LineEventValue::constant(255.0),
            start_beat: Beat::ZERO,
            end_beat: Beat::new(0, Rational32::new(1, 4)),
        })],
        ..Default::default()
    }
}

pub(super) fn create_line_system(mut edits: Edits) -> Result {
    let line = new_line();
    edits.once(t!("history.create_line"), move |commands| {
        spawn_line(line, commands, None);
    });
    Ok(())
}

pub(super) fn create_line_from_selected_system(
    selected_line: Res<SelectedLine>,
    index: Res<EventIndex>,
    pending: Query<(), With<Pending>>,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
    mut edits: Edits,
) -> Result {
    let beat = bpm_list.beat_at(time.0).value();
    let events = LineEventKind::ALL
        .into_iter()
        .filter_map(|kind| {
            let default = kind.default_value();
            let value = index
                .until(selected_line.0, kind, beat)
                .find(|(entity, _)| !pending.contains(*entity))
                .map_or(default, |(_, event)| event.value_at(beat));
            (value != default).then(|| {
                Identified::new(LineEvent {
                    kind,
                    value: LineEventValue::constant(value),
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                })
            })
        })
        .collect();
    let new_line = SerializedLine {
        events,
        ..Default::default()
    };
    edits.once(t!("history.create_line"), move |commands| {
        spawn_line(new_line, commands, None);
    });
    Ok(())
}
