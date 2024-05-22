use crate::chart::note::{Note, NoteBundle};
use bevy::app::{App, Update};
use bevy::prelude::{Entity, World};
use undo::Edit;

pub struct CreateNote(pub Note, pub Option<Entity>);

impl CreateNote {
    #[allow(dead_code)]
    pub fn new(note: Note) -> Self {
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

pub struct EditNote {
    entity: Entity,
    from: Note,
    to: Note,
}

impl EditNote {
    fn new(entity: Entity, from: Note, to: Note) -> Self {
        Self { entity, from, to }
    }
}

impl Edit for EditNote {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some(mut note) = target.entity_mut(self.entity).get_mut::<Note>() {
            *note = self.to;
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some(mut note) = target.entity_mut(self.entity).get_mut::<Note>() {
            *note = self.from;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::beat::Beat;
    use crate::chart::note::{Note, NoteBundle, NoteKind};
    use crate::editing::command::EditorCommand;
    use undo::History;

    fn test_remove_note_system(world: &mut World) {
        let mut history = History::new();
        let note = Note::new(NoteKind::Tap, true, Beat::ZERO, 0.0, 1.0);
        let entity = world.spawn(NoteBundle::new(note)).id();
        assert!(world.query::<&Note>().get_single(world).is_ok());
        history.edit(world, EditorCommand::RemoveNote(RemoveNote(entity, note)));
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
        history.edit(world, EditorCommand::CreateNote(CreateNote::new(note)));
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
