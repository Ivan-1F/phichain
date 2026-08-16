use bevy::prelude::{Children, Entity, World};

/// Replace the given entity with an empty one. Removes all its children and components
pub fn replace_with_empty(world: &mut World, entity: Entity) {
    // despawn all children
    if let Some(children) = world.entity_mut(entity).take::<Children>() {
        for child in children.iter() {
            // linked_spawn cascades may have already despawned some children
            if let Ok(child) = world.get_entity_mut(*child) {
                child.despawn();
            }
        }
    }

    // remove all components
    world.entity_mut(entity).retain::<()>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::ChildOf;
    use phichain_chart::beat;
    use phichain_chart::curve_note_track::CurveNoteTrackOptions;
    use phichain_chart::id::{CurveNoteTrackId, NoteId};
    use phichain_chart::line::Line;
    use phichain_chart::note::{Note, NoteKind};
    use phichain_game::curve_note_track::{CurveNoteTrackFrom, CurveNoteTrackTo};

    /// A note despawned inside `replace_with_empty` cascades to tracks
    /// referencing it (linked_spawn); the cleanup loop must tolerate the
    /// track already being gone instead of panicking
    #[test]
    fn tolerates_children_killed_by_linked_spawn_cascade() {
        let mut world = World::new();
        let line = world.spawn(Line::default()).id();
        let note = world
            .spawn((
                NoteId::new(),
                Note::new(NoteKind::Tap, true, beat!(0), 0.0, 1.0),
                ChildOf(line),
            ))
            .id();
        let cnt = world
            .spawn((
                CurveNoteTrackOptions::default(),
                CurveNoteTrackId::new(),
                CurveNoteTrackFrom(note),
                CurveNoteTrackTo(note),
                ChildOf(line),
            ))
            .id();

        replace_with_empty(&mut world, line);

        assert!(world.get_entity(line).is_ok());
        assert!(world.get_entity(note).is_err());
        assert!(world.get_entity(cnt).is_err());
    }
}
