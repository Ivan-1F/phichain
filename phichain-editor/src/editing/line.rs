use crate::action::ActionRegistrationExt;
use crate::editing::history::Edits;
use crate::hotkey::modifier::Modifier;
use crate::hotkey::Hotkey;
use crate::selection::SelectedLine;
use crate::timing::ChartTime;
use bevy::prelude::*;
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
use phichain_chart::id::{Identified, LineId};
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

fn create_line_system(mut edits: Edits) -> Result {
    edits.once(t!("history.create_line"), |commands| {
        spawn_line(SerializedLine::default(), commands, None);
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
    let events = [
        (LineEventKind::X, 0.0),
        (LineEventKind::Y, 0.0),
        (LineEventKind::Rotation, 0.0),
        (LineEventKind::Opacity, 0.0),
        (LineEventKind::Speed, 10.0),
    ]
    .into_iter()
    .map(|(kind, fallback)| {
        let value = index
            .until(selected_line.0, kind, beat)
            .find(|(entity, _)| !pending.contains(*entity))
            .map(|(_, event)| event.value_at(beat))
            .unwrap_or(fallback);
        Identified::new(LineEvent {
            kind,
            value: LineEventValue::constant(value),
            start_beat: Beat::ZERO,
            end_beat: Beat::ONE,
        })
    })
    .collect();
    let new_line = SerializedLine {
        line: Default::default(),
        id: LineId::new(),
        notes: vec![],
        events,
        children: vec![],
        curve_note_tracks: vec![],
    };
    edits.once(t!("history.create_line"), move |commands| {
        spawn_line(new_line, commands, None);
    });
    Ok(())
}
