use crate::action::ActionRegistrationExt;
use crate::editing::history::{ChangeSet, Edits};
use crate::hotkey::Hotkey;
use crate::selection::Selected;
use bevy::prelude::*;

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
    selected: Query<Entity, With<Selected>>,
    mut edits: Edits,
) -> Result {
    let targets: Vec<Entity> = selected.iter().collect();
    if !targets.is_empty() {
        edits.once(
            |changes: &ChangeSet| {
                t!(
                    "history.delete",
                    objects = changes.destroyed_counts().text()
                )
                .into_owned()
            },
            move |commands| {
                for entity in targets {
                    commands.entity(entity).try_despawn();
                }
            },
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editing::history::{open_document, EditorHistory, HistoryPlugin};
    use crate::id_index::IdIndexPlugin;
    use bevy::ecs::system::RunSystemOnce;
    use phichain_chart::beat::Beat;
    use phichain_chart::line::Line;
    use phichain_chart::note::{Note, NoteKind};
    use phichain_game::curve_note_track::{CurveNoteTrackFrom, CurveNoteTrackTo};

    #[test]
    fn deleting_a_note_counts_the_track_it_cascades_to() {
        let mut app = App::new();
        app.add_plugins((IdIndexPlugin, HistoryPlugin));
        let world = app.world_mut();
        let line = world.spawn(Line::default()).id();
        let note = Note::new(NoteKind::Tap, true, Beat::from(1.0), 0.0, 1.0);
        let a = world.spawn((note, ChildOf(line), Selected)).id();
        let b = world.spawn((note, ChildOf(line))).id();
        let track = world
            .spawn((CurveNoteTrackFrom(a), CurveNoteTrackTo(b), ChildOf(line)))
            .id();
        open_document(world);

        world
            .run_system_once::<_, Result, _>(delete_selected_system)
            .unwrap()
            .unwrap();
        assert!(world.get_entity(a).is_err());
        assert!(world.get_entity(track).is_err());
        assert!(world.get_entity(b).is_ok());
        assert_eq!(
            world.resource::<EditorHistory>().undo_label().as_deref(),
            Some("Delete 1 note, 1 curve track")
        );

        world.resource_scope(|world, mut history: Mut<EditorHistory>| history.undo(world));
        assert!(world.get_entity(a).is_err());
        assert_eq!(world.query::<&Note>().iter(world).count(), 2);
        assert_eq!(world.query::<&CurveNoteTrackTo>().iter(world).count(), 1);
    }
}
