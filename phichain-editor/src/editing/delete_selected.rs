use crate::action::ActionRegistrationExt;
use crate::editing::description::ObjectCounts;
use crate::editing::history::Edits;
use crate::hotkey::Hotkey;
use crate::selection::Selected;
use bevy::prelude::*;
use phichain_chart::event::LineEvent;
use phichain_chart::note::Note;
use phichain_game::curve_note_track::CurveNoteTrackTo;
use phichain_game::Pending;

pub struct DeleteSelectedPlugin;

impl Plugin for DeleteSelectedPlugin {
    fn build(&self, app: &mut App) {
        app.add_action(
            "phichain.delete_selected",
            delete_selected_system,
            Some(Hotkey::new(KeyCode::Backspace, vec![])),
        );
    }
}

pub(super) fn delete_selected_system(
    selected: Query<
        (
            Entity,
            Has<Pending>,
            Has<Note>,
            Has<LineEvent>,
            Has<CurveNoteTrackTo>,
        ),
        With<Selected>,
    >,
    mut commands: Commands,
    mut edits: Edits,
) -> Result {
    let mut targets = Vec::new();
    let mut counts = ObjectCounts::default();
    for (entity, pending, note, event, track) in &selected {
        if pending {
            commands.entity(entity).try_despawn();
        } else {
            targets.push(entity);
            counts.notes += usize::from(note);
            counts.events += usize::from(event);
            counts.tracks += usize::from(track);
        }
    }
    if !targets.is_empty() {
        edits.once(
            t!("history.delete", objects = counts.text()),
            move |commands| {
                for entity in targets {
                    commands.entity(entity).try_despawn();
                }
            },
        );
    }
    Ok(())
}
