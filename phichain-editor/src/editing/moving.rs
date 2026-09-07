use crate::action::ActionRegistrationExt;
use crate::editing::history::Edits;
use crate::hotkey::Hotkey;
use crate::selection::Selected;
use crate::timeline::settings::TimelineSettings;
use bevy::prelude::*;
use phichain_chart::beat::Beat;
use phichain_chart::event::LineEvent;
use phichain_chart::note::Note;
use phichain_game::{Derived, Pending};

type SelectedNotes<'w, 's> =
    Query<'w, 's, (Entity, &'static Note), (With<Selected>, Without<Derived>, Without<Pending>)>;
type SelectedEvents<'w, 's> =
    Query<'w, 's, (Entity, &'static LineEvent), (With<Selected>, Without<Pending>)>;

pub struct MovingPlugin;

impl Plugin for MovingPlugin {
    fn build(&self, app: &mut App) {
        app.add_action(
            "phichain.move_up",
            move_up_system,
            Some(Hotkey::new(KeyCode::ArrowUp, vec![])),
        )
        .add_action(
            "phichain.move_down",
            move_down_system,
            Some(Hotkey::new(KeyCode::ArrowDown, vec![])),
        )
        .add_action(
            "phichain.move_left",
            move_left_system,
            Some(Hotkey::new(KeyCode::ArrowLeft, vec![])),
        )
        .add_action(
            "phichain.move_right",
            move_right_system,
            Some(Hotkey::new(KeyCode::ArrowRight, vec![])),
        );
    }
}

fn move_vertical(
    settings: &TimelineSettings,
    notes: &SelectedNotes,
    events: &SelectedEvents,
    edits: &mut Edits,
    forward: bool,
) {
    let start = notes
        .iter()
        .map(|(_, note)| note.beat)
        .chain(events.iter().map(|(_, event)| event.start_beat))
        .min();
    let Some(start) = start else {
        return;
    };
    let step = settings.minimum_beat();
    let to = settings
        .attach((if forward { start + step } else { start - step }).value())
        .max(Beat::ZERO);
    let delta = to - start;
    let notes: Vec<_> = notes
        .iter()
        .map(|(entity, note)| {
            (
                entity,
                Note {
                    beat: note.beat + delta,
                    ..*note
                },
            )
        })
        .collect();
    let events: Vec<_> = events
        .iter()
        .map(|(entity, event)| {
            (
                entity,
                LineEvent {
                    start_beat: event.start_beat + delta,
                    end_beat: event.end_beat + delta,
                    ..*event
                },
            )
        })
        .collect();
    edits.once(
        t!("history.move_objects", count = notes.len() + events.len()),
        move |commands| {
            for (entity, note) in notes {
                commands.entity(entity).insert(note);
            }
            for (entity, event) in events {
                commands.entity(entity).insert(event);
            }
        },
    );
}

fn move_up_system(
    settings: Res<TimelineSettings>,
    notes: SelectedNotes,
    events: SelectedEvents,
    mut edits: Edits,
) -> Result {
    move_vertical(&settings, &notes, &events, &mut edits, true);
    Ok(())
}
fn move_down_system(
    settings: Res<TimelineSettings>,
    notes: SelectedNotes,
    events: SelectedEvents,
    mut edits: Edits,
) -> Result {
    move_vertical(&settings, &notes, &events, &mut edits, false);
    Ok(())
}
fn move_horizontal(
    settings: &TimelineSettings,
    notes: &SelectedNotes,
    edits: &mut Edits,
    right: bool,
) {
    let edge = notes
        .iter()
        .map(|(_, note)| note.x)
        .reduce(|a, b| if right { a.max(b) } else { a.min(b) });
    let Some(edge) = edge else {
        return;
    };
    let step = settings.minimum_lane() * if right { 1.0 } else { -1.0 };
    let delta = settings.attach_x(edge + step) - edge;
    let notes: Vec<_> = notes
        .iter()
        .map(|(entity, note)| {
            (
                entity,
                Note {
                    x: note.x + delta,
                    ..*note
                },
            )
        })
        .collect();
    edits.once(
        t!("history.move_objects", count = notes.len()),
        move |commands| {
            for (entity, note) in notes {
                commands.entity(entity).insert(note);
            }
        },
    );
}
fn move_left_system(
    settings: Res<TimelineSettings>,
    notes: SelectedNotes,
    mut edits: Edits,
) -> Result {
    move_horizontal(&settings, &notes, &mut edits, false);
    Ok(())
}
fn move_right_system(
    settings: Res<TimelineSettings>,
    notes: SelectedNotes,
    mut edits: Edits,
) -> Result {
    move_horizontal(&settings, &notes, &mut edits, true);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editing::history::{open_document, EditorHistory, HistoryPlugin};
    use crate::id_index::IdIndexPlugin;
    use bevy::ecs::system::RunSystemOnce;
    use phichain_chart::event::{LineEventKind, LineEventValue};
    use phichain_chart::note::NoteKind;

    #[test]
    fn mixed_selection_moves_as_one_edit_and_a_blocked_move_preserves_redo() {
        let mut app = App::new();
        app.add_plugins((IdIndexPlugin, HistoryPlugin))
            .init_resource::<TimelineSettings>();
        let world = app.world_mut();
        let note = Note::new(NoteKind::Tap, true, Beat::ZERO, 0.0, 1.0);
        let event = LineEvent {
            kind: LineEventKind::X,
            start_beat: Beat::from(2.0),
            end_beat: Beat::from(5.0),
            value: LineEventValue::constant(1.0),
        };
        let note_entity = world.spawn((note, Selected)).id();
        let event_entity = world.spawn((event, Selected)).id();
        let preview = world.spawn((note, Selected, Pending)).id();
        let generated = world.spawn((note, Selected, Derived)).id();
        open_document(world);

        world
            .run_system_once::<_, Result, _>(move_up_system)
            .unwrap()
            .unwrap();
        let step = world.resource::<TimelineSettings>().minimum_beat();
        assert_eq!(world.get::<Note>(note_entity).unwrap().beat, step);
        assert_eq!(
            world.get::<LineEvent>(event_entity).unwrap().start_beat,
            event.start_beat + step
        );
        assert_eq!(
            world.get::<LineEvent>(event_entity).unwrap().end_beat,
            event.end_beat + step
        );
        assert_eq!(world.get::<Note>(preview), Some(&note));
        assert_eq!(world.get::<Note>(generated), Some(&note));

        world
            .resource_scope(|world, mut history: Mut<EditorHistory>| history.undo(world))
            .unwrap();
        assert_eq!(world.get::<Note>(note_entity), Some(&note));
        assert_eq!(world.get::<LineEvent>(event_entity), Some(&event));
        assert!(world.resource::<EditorHistory>().is_saved());

        world
            .run_system_once::<_, Result, _>(move_down_system)
            .unwrap()
            .unwrap();
        assert!(world.resource::<EditorHistory>().is_saved());
        world
            .resource_scope(|world, mut history: Mut<EditorHistory>| history.redo(world))
            .unwrap();
        assert_eq!(world.get::<Note>(note_entity).unwrap().beat, step);
        assert_eq!(
            world.get::<LineEvent>(event_entity).unwrap().start_beat,
            event.start_beat + step
        );
    }
}
