use crate::chart::note::{Note, NoteBundle};
use bevy::prelude::*;
use undo::Edit;

pub struct CreateNote(pub Note, pub Option<Entity>);

impl CreateNote {
    #[allow(dead_code)]
    fn new(note: Note) -> Self {
        Self(note, None)
    }
}

impl Edit for CreateNote {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        self.1 = Some(target.spawn(NoteBundle::new(self.0)).id());
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some(entity) = self.1 {
            target.despawn(entity);
        }
    }
}

pub struct RemoveNote(pub Entity, pub Note);

impl Edit for RemoveNote {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        target.despawn(self.0);
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        self.0 = target.spawn(NoteBundle::new(self.1)).id();
    }
}

pub struct Nop;

impl Edit for Nop {
    type Target = ();
    type Output = ();

    fn edit(&mut self, _target: &mut Self::Target) -> Self::Output {}

    fn undo(&mut self, _target: &mut Self::Target) -> Self::Output {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::beat::Beat;
    use crate::chart::note::{Note, NoteBundle, NoteKind};
    use undo::History;

    fn test_remove_note_system(world: &mut World) {
        let mut history = History::new();
        let note = Note::new(NoteKind::Tap, true, Beat::ZERO, 0.0, 1.0);
        let entity = world.spawn(NoteBundle::new(note)).id();
        assert!(world.query::<&Note>().get_single(world).is_ok());
        history.edit(world, RemoveNote(entity, note));
        assert!(world.query::<&Note>().get_single(world).is_err());
        history.undo(world);
        assert!(world.query::<&Note>().get_single(world).is_ok());
        history.redo(world);
        assert!(world.query::<&Note>().get_single(world).is_err());
    }

    #[test]
    fn test_remove_note_command() {
        let mut app = App::new();
        app.add_systems(Update, test_remove_note_system);
        app.update();
    }

    fn test_create_note_system(world: &mut World) {
        let mut history = History::new();
        let note = Note::new(NoteKind::Tap, true, Beat::ZERO, 0.0, 1.0);
        assert!(world.query::<&Note>().get_single(world).is_err());
        history.edit(world, CreateNote::new(note));
        assert!(world.query::<&Note>().get_single(world).is_ok());
        history.undo(world);
        assert!(world.query::<&Note>().get_single(world).is_err());
        history.redo(world);
        assert!(world.query::<&Note>().get_single(world).is_ok());
    }

    #[test]
    fn test_create_note_command() {
        let mut app = App::new();
        app.add_systems(Update, test_create_note_system);
        app.update();
    }
}
