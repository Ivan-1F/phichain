use bevy::prelude::*;
use phichain_chart::note::{Note, NoteBundle};
use undo::Edit;

#[derive(Debug, Copy, Clone)]
pub struct CreateNote {
    pub line_entity: Entity,
    pub note: Note,
    pub note_entity: Option<Entity>,
}

impl CreateNote {
    pub fn new(line: Entity, note: Note) -> Self {
        Self {
            line_entity: line,
            note,
            note_entity: None,
        }
    }
}

impl Edit for CreateNote {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        target.entity_mut(self.line_entity).with_children(|parent| {
            self.note_entity = Some(parent.spawn(NoteBundle::new(self.note)).id());
        });
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some(entity) = self.note_entity {
            target.entity_mut(entity).despawn_recursive();
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RemoveNote {
    pub entity: Entity,
    pub note: Option<Note>,
}

impl RemoveNote {
    pub fn new(entity: Entity) -> Self {
        Self { entity, note: None }
    }
}

impl Edit for RemoveNote {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        self.note = target.entity(self.entity).get::<Note>().copied();
        target.entity_mut(self.entity).retain::<Parent>();
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some(note) = self.note {
            target.entity_mut(self.entity).insert(NoteBundle::new(note));
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct EditNote {
    entity: Entity,
    from: Note,
    to: Note,
}

impl EditNote {
    pub fn new(entity: Entity, from: Note, to: Note) -> Self {
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
    use crate::editing::command::EditorCommand;
    use phichain_chart::beat::Beat;
    use phichain_chart::line::LineBundle;
    use phichain_chart::note::{Note, NoteBundle, NoteKind};
    use undo::History;

    fn test_remove_note_system(world: &mut World) {
        let mut history = History::new();

        let line = world.spawn(LineBundle::default()).id();
        let note = Note::new(NoteKind::Tap, true, Beat::ZERO, 0.0, 1.0);
        let entity = world.spawn(NoteBundle::new(note)).id();
        world.entity_mut(line).add_child(entity);

        assert!(world.query::<&Note>().get_single(world).is_ok());
        history.edit(world, EditorCommand::RemoveNote(RemoveNote::new(entity)));
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
        let line = world.spawn(LineBundle::default()).id();
        let note = Note::new(NoteKind::Tap, true, Beat::ZERO, 0.0, 1.0);
        assert!(world.query::<&Note>().get_single(world).is_err());
        history.edit(
            world,
            EditorCommand::CreateNote(CreateNote::new(line, note)),
        );
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
