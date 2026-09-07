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
use phichain_game::event::Events;
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

fn create_line_from_selected_system(
    selected_line: Res<SelectedLine>,
    event_query: Query<&LineEvent, Without<Pending>>,
    events_query: Query<&Events>,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
    mut edits: Edits,
) -> Result {
    let beat = bpm_list.beat_at(time.0).value();
    let events = events_query.get(selected_line.0).ok();
    let events = [
        (LineEventKind::X, 0.0),
        (LineEventKind::Y, 0.0),
        (LineEventKind::Rotation, 0.0),
        (LineEventKind::Opacity, 0.0),
        (LineEventKind::Speed, 10.0),
    ]
    .into_iter()
    .map(|(kind, fallback)| {
        let value = events
            .iter()
            .flat_map(|events| events.iter())
            .filter_map(|entity| event_query.get(entity).ok())
            .filter(|event| event.kind == kind)
            .map(|event| event.evaluate_inclusive(beat))
            .max()
            .and_then(|result| result.value())
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
