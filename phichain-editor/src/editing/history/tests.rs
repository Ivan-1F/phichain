use super::*;
use crate::editing::pending::Pending;
use crate::id_index::IdIndexPlugin;
use crate::selection::{Selected, SelectedLine};
use bevy::ecs::system::RunSystemOnce;
use bevy::ecs::system::SystemState;
use phichain_chart::beat::Beat;
use phichain_chart::line::Line;
use phichain_chart::note::NoteKind;
use phichain_game::Derived;

fn fixture() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((IdIndexPlugin, HistoryPlugin));
    let line = app.world_mut().spawn(Line::default()).id();
    app.world_mut().insert_resource(SelectedLine(line));
    open_document(app.world_mut());
    (app, line)
}

fn note() -> Note {
    Note::new(NoteKind::Tap, true, Beat::ONE, 100.0, 1.0)
}

fn create(app: &mut App, line: Entity) -> (Entity, NoteId) {
    apply_edit(app.world_mut(), "create".into(), |commands| {
        commands.spawn((note(), ChildOf(line)));
    });
    let (entity, id) = app
        .world_mut()
        .query::<(Entity, &NoteId)>()
        .single(app.world())
        .unwrap();
    (entity, *id)
}

fn undo(app: &mut App) {
    app.world_mut()
        .resource_scope(|world, mut history: Mut<EditorHistory>| history.undo(world))
        .unwrap();
}

fn redo(app: &mut App) {
    app.world_mut()
        .resource_scope(|world, mut history: Mut<EditorHistory>| history.redo(world))
        .unwrap();
}

fn entity_with_id(app: &mut App, id: NoteId) -> Entity {
    app.world_mut()
        .query::<(Entity, &NoteId)>()
        .iter(app.world())
        .find(|(_, candidate)| **candidate == id)
        .unwrap()
        .0
}

#[test]
fn create_undo_redo_restores_identity_parent_and_required_components() {
    let (mut app, line) = fixture();
    let (original, id) = create(&mut app, line);
    assert_eq!(app.world().resource::<EditorHistory>().record.len(), 1);
    undo(&mut app);
    assert!(app.world().get_entity(original).is_err());
    assert!(app.world().resource::<EditorHistory>().is_saved());
    redo(&mut app);
    let restored = entity_with_id(&mut app, id);
    assert_ne!(restored, original);
    assert_eq!(app.world().get::<Note>(restored), Some(&note()));
    assert_eq!(app.world().get::<ChildOf>(restored).unwrap().parent(), line);
    assert!(app.world().get::<Sprite>(restored).is_some());
    assert!(app
        .world()
        .get::<Children>(line)
        .unwrap()
        .contains(&restored));
    assert_eq!(app.world().resource::<EditorHistory>().record.len(), 1);
}

#[test]
fn repeated_replacements_keep_the_first_value_and_redo_does_not_run_the_action() {
    let (mut app, line) = fixture();
    let (entity, _) = create(&mut app, line);
    apply_edit(app.world_mut(), "edit".into(), |commands| {
        commands.entity(entity).insert(Note { x: 120.0, ..note() });
        commands.entity(entity).insert(Note {
            x: 150.0,
            above: false,
            ..note()
        });
    });
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 150.0);
    undo(&mut app);
    assert_eq!(app.world().get::<Note>(entity), Some(&note()));
    redo(&mut app);
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 150.0);
    assert!(!app.world().get::<Note>(entity).unwrap().above);
    assert_eq!(app.world().resource::<EditorHistory>().record.len(), 2);
}

#[test]
fn undo_delete_restores_an_unselected_note_and_preserves_current_selection() {
    let mut app = App::new();
    app.add_plugins((IdIndexPlugin, HistoryPlugin))
        .init_resource::<crate::notification::ToastsStorage>();
    let line = app.world_mut().spawn(Line::default()).id();
    let other_line = app.world_mut().spawn(Line::default()).id();
    app.world_mut().insert_resource(SelectedLine(line));
    open_document(app.world_mut());
    let id = NoteId::new();
    let other_id = NoteId::new();
    apply_edit(app.world_mut(), "create notes".into(), |commands| {
        commands.spawn((note(), id, ChildOf(line), Selected));
        commands.spawn((note(), other_id, ChildOf(other_line)));
    });
    let entity = entity_with_id(&mut app, id);
    let other = entity_with_id(&mut app, other_id);
    app.world_mut()
        .run_system_once::<_, Result, _>(crate::editing::delete_selected::delete_selected_system)
        .unwrap()
        .unwrap();
    app.world_mut().flush();
    assert!(app.world().get_entity(entity).is_err());
    app.world_mut().entity_mut(other).insert(Selected);
    app.world_mut().resource_mut::<SelectedLine>().0 = other_line;
    undo(&mut app);
    let restored = entity_with_id(&mut app, id);
    assert_ne!(restored, entity);
    assert_eq!(app.world().get::<Note>(restored), Some(&note()));
    assert_eq!(app.world().get::<ChildOf>(restored).unwrap().parent(), line);
    assert!(app.world().get::<Selected>(restored).is_none());
    assert!(app.world().get::<Selected>(other).is_some());
    assert_eq!(app.world().resource::<SelectedLine>().0, other_line);
    redo(&mut app);
    assert!(app.world().get_entity(restored).is_err());
    assert!(app.world().get::<Selected>(other).is_some());
    assert_eq!(app.world().resource::<SelectedLine>().0, other_line);
}

#[test]
fn empty_edits_preserve_redo_and_only_successful_changes_replace_the_tail() {
    let (mut app, line) = fixture();
    let (entity, _) = create(&mut app, line);
    apply_edit(app.world_mut(), "edit".into(), |commands| {
        commands.entity(entity).insert(Note {
            above: false,
            ..note()
        });
    });
    let old_head = app.world().resource::<EditorHistory>().head();
    undo(&mut app);
    apply_edit(app.world_mut(), "no change".into(), |commands| {
        commands.entity(entity).insert(note());
    });
    assert!(app.world().resource::<EditorHistory>().record.can_redo());
    apply_edit(app.world_mut(), "create then delete".into(), |commands| {
        let temporary = commands.spawn((note(), ChildOf(line))).id();
        commands.entity(temporary).despawn();
    });
    assert!(app.world().resource::<EditorHistory>().record.can_redo());
    apply_edit(app.world_mut(), "another edit".into(), |commands| {
        commands.entity(entity).insert(Note { x: 200.0, ..note() });
    });
    let history = app.world().resource::<EditorHistory>();
    assert!(!history.record.can_redo());
    assert_eq!(history.record.len(), 2);
    assert_ne!(
        history.head(),
        old_head,
        "a reused index must have a new state identity"
    );
}

#[test]
fn loading_preview_and_derived_entities_do_not_create_history() {
    let (mut app, line) = fixture();
    // Runtime and preview entities can change outside an edit.
    let preview = app.world_mut().spawn((note(), Pending, ChildOf(line))).id();
    app.world_mut()
        .entity_mut(preview)
        .insert(Note { x: 50.0, ..note() });
    app.world_mut().entity_mut(preview).despawn();
    let derived = app.world_mut().spawn((note(), Derived, ChildOf(line))).id();
    app.world_mut().entity_mut(derived).despawn();
    assert_eq!(app.world().resource::<EditorHistory>().record.len(), 0);
    close_document(app.world_mut());
    app.world_mut().entity_mut(line).despawn();
    let new_line = app.world_mut().spawn(Line::default()).id();
    app.world_mut().spawn((note(), ChildOf(new_line)));
    open_document(app.world_mut());
    assert!(app.world().resource::<EditorHistory>().is_saved());
    assert!(!app.world().resource::<EditorHistory>().record.can_undo());
}

#[test]
fn edit_commands_are_deferred_grouped_and_keep_queue_order() {
    let (mut app, line) = fixture();
    let (entity, _) = create(&mut app, line);
    let system = app.world_mut().register_system(move |mut edits: Edits| {
        edits.once("first", move |commands| {
            commands.entity(entity).insert(Note { x: 200.0, ..note() });
        });
        edits.once("second", move |commands| {
            commands.entity(entity).insert(Note { x: 300.0, ..note() });
        });
    });
    app.world_mut().run_system(system).unwrap();
    app.world_mut().flush();
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 300.0);
    undo(&mut app);
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 200.0);
    undo(&mut app);
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 100.0);
}

#[test]
fn missing_parent_is_rejected_before_moving_history_cursor() {
    let (mut app, line) = fixture();
    let (_, id) = create(&mut app, line);
    undo(&mut app);
    // Simulate an external document inconsistency, rather than a supported edit.
    app.world_mut().resource_mut::<Recorder>().enabled = false;
    app.world_mut().entity_mut(line).despawn();
    let before = app.world().resource::<EditorHistory>().head();
    let result = app
        .world_mut()
        .resource_scope(|world, mut history: Mut<EditorHistory>| history.redo(world));
    assert!(result.is_err());
    assert_eq!(app.world().resource::<EditorHistory>().head(), before);
    assert!(app.world().resource::<EditorHistory>().record.can_redo());
    assert!(!app
        .world_mut()
        .query::<&NoteId>()
        .iter(app.world())
        .any(|candidate| *candidate == id));
}

#[test]
fn bounded_history_and_saved_state_survive_eviction() {
    let (mut app, line) = fixture();
    app.world_mut().resource_mut::<EditorHistory>().record = Record::builder().limit(2).build();
    let (entity, _) = create(&mut app, line);
    app.world_mut().resource_mut::<EditorHistory>().set_saved();
    for x in [200.0, 300.0] {
        apply_edit(app.world_mut(), "edit".into(), |commands| {
            commands.entity(entity).insert(Note { x, ..note() });
        });
    }
    assert_eq!(app.world().resource::<EditorHistory>().record.len(), 2);
    undo(&mut app);
    undo(&mut app);
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 100.0);
    assert!(app.world().resource::<EditorHistory>().is_saved());
    assert!(!app.world().resource::<EditorHistory>().record.can_undo());
}

#[test]
fn batch_delete_is_one_entry_and_earlier_edits_follow_restored_entities() {
    let (mut app, line) = fixture();
    app.init_resource::<crate::notification::ToastsStorage>();
    apply_edit(app.world_mut(), "create two".into(), |commands| {
        commands.spawn((note(), ChildOf(line), Selected));
        commands.spawn((Note { x: 200.0, ..note() }, ChildOf(line), Selected));
    });
    let originals: Vec<_> = app
        .world_mut()
        .query::<(Entity, &NoteId)>()
        .iter(app.world())
        .map(|(entity, id)| (entity, *id))
        .collect();
    let (first, first_id) = originals[0];
    apply_edit(app.world_mut(), "modify".into(), |commands| {
        commands.entity(first).insert(Note { x: 300.0, ..note() });
    });
    app.world_mut()
        .run_system_once::<_, Result, _>(crate::editing::delete_selected::delete_selected_system)
        .unwrap()
        .unwrap();
    assert_eq!(app.world().resource::<EditorHistory>().record.len(), 3);
    for (entity, _) in &originals {
        assert!(app.world().get_entity(*entity).is_err());
    }
    undo(&mut app);
    for (_, id) in &originals {
        let restored = entity_with_id(&mut app, *id);
        assert!(app.world().get::<Selected>(restored).is_none());
    }
    let restored = entity_with_id(&mut app, first_id);
    assert_ne!(restored, first);
    undo(&mut app);
    assert_ne!(app.world().get::<Note>(restored).unwrap().x, 300.0);
    undo(&mut app);
    assert_eq!(
        app.world_mut().query::<&Note>().iter(app.world()).count(),
        0
    );
    redo(&mut app);
    redo(&mut app);
    let restored = entity_with_id(&mut app, first_id);
    assert_eq!(app.world().get::<Note>(restored).unwrap().x, 300.0);
    redo(&mut app);
    assert_eq!(
        app.world_mut().query::<&Note>().iter(app.world()).count(),
        0
    );
}

#[test]
fn batch_delete_with_hold_rebuilds_render_children_on_undo() {
    use phichain_game::core::{spawn_hold_component_system, HoldHead, HoldTail};

    let (mut app, line) = fixture();
    app.init_resource::<crate::notification::ToastsStorage>()
        .add_systems(Update, spawn_hold_component_system);
    let hold = Note {
        kind: NoteKind::Hold {
            hold_beat: Beat::ONE,
        },
        ..note()
    };
    let hold_id = NoteId::new();
    let tap_id = NoteId::new();
    apply_edit(app.world_mut(), "create tap and hold".into(), |commands| {
        commands.spawn((note(), tap_id, ChildOf(line), Selected));
        commands.spawn((hold, hold_id, ChildOf(line), Selected));
    });
    // Use the real rendering producer: a bare ECS fixture would miss the
    // children that distinguish a rendered Hold from other notes.
    app.update();
    let hold_entity = entity_with_id(&mut app, hold_id);
    let original_children: Vec<_> = app
        .world()
        .get::<Children>(hold_entity)
        .unwrap()
        .iter()
        .collect();
    assert_eq!(original_children.len(), 2);
    app.world_mut()
        .run_system_once::<_, Result, _>(crate::editing::delete_selected::delete_selected_system)
        .unwrap()
        .unwrap();
    assert!(
        app.world().get_entity(hold_entity).is_err(),
        "render children must not prevent deleting a Hold"
    );
    for child in original_children {
        assert!(app.world().get_entity(child).is_err());
    }
    assert_eq!(app.world().resource::<EditorHistory>().record.len(), 2);

    for _ in 0..2 {
        undo(&mut app);
        let hold_entity = entity_with_id(&mut app, hold_id);
        let tap_entity = entity_with_id(&mut app, tap_id);
        assert_eq!(app.world().get::<Note>(hold_entity), Some(&hold));
        assert_eq!(app.world().get::<Note>(tap_entity), Some(&note()));
        assert!(app.world().get::<Selected>(hold_entity).is_none());
        assert!(app.world().get::<Selected>(tap_entity).is_none());
        // History restores the note; its visual children come from the producer.
        assert!(app.world().get::<Children>(hold_entity).is_none());
        app.update();
        let children = app.world().get::<Children>(hold_entity).unwrap();
        assert_eq!(children.len(), 2);
        assert!(children
            .iter()
            .any(|child| app.world().get::<HoldHead>(child).is_some()));
        assert!(children
            .iter()
            .any(|child| app.world().get::<HoldTail>(child).is_some()));
        for child in children {
            assert!(app.world().get::<Derived>(*child).is_some());
        }
        assert_eq!(
            app.world().get::<ChildOf>(hold_entity).unwrap().parent(),
            line
        );
        assert_eq!(app.world().resource::<EditorHistory>().record.len(), 2);
        redo(&mut app);
        app.update();
        assert_eq!(
            app.world_mut().query::<&Note>().iter(app.world()).count(),
            0
        );
        assert_eq!(
            app.world_mut()
                .query::<&HoldHead>()
                .iter(app.world())
                .count(),
            0
        );
        assert_eq!(
            app.world_mut()
                .query::<&HoldTail>()
                .iter(app.world())
                .count(),
            0
        );
    }
}

#[test]
fn mixed_and_curve_endpoint_selections_are_rejected_as_a_whole() {
    use phichain_game::curve_note_track::{CurveNoteTrackFrom, CurveNoteTrackTo};
    let (mut app, line) = fixture();
    app.init_resource::<crate::notification::ToastsStorage>();
    let (entity, _) = create(&mut app, line);
    app.world_mut().entity_mut(entity).insert(Selected);
    app.world_mut().entity_mut(line).insert(Selected);
    app.world_mut()
        .run_system_once::<_, Result, _>(crate::editing::delete_selected::delete_selected_system)
        .unwrap()
        .unwrap();
    assert!(app.world().get::<Note>(entity).is_some());
    assert_eq!(app.world().resource::<EditorHistory>().record.len(), 1);
    app.world_mut().entity_mut(line).remove::<Selected>();
    // Load an ordinary second note and a curve track before recording resumes.
    close_document(app.world_mut());
    app.world_mut().entity_mut(line).despawn();
    let line = app.world_mut().spawn(Line::default()).id();
    let from = app
        .world_mut()
        .spawn((note(), ChildOf(line), Selected))
        .id();
    let to = app.world_mut().spawn((note(), ChildOf(line))).id();
    let unrelated = app
        .world_mut()
        .spawn((note(), ChildOf(line), Selected))
        .id();
    let track = app
        .world_mut()
        .spawn((
            CurveNoteTrackFrom(from),
            CurveNoteTrackTo(to),
            ChildOf(line),
        ))
        .id();
    open_document(app.world_mut());
    app.world_mut()
        .run_system_once::<_, Result, _>(crate::editing::delete_selected::delete_selected_system)
        .unwrap()
        .unwrap();
    for entity in [from, to, unrelated, track] {
        assert!(app.world().get_entity(entity).is_ok());
    }
    assert_eq!(app.world().resource::<EditorHistory>().record.len(), 0);
}

#[test]
fn lifecycle_observer_commands_belong_to_the_edit_that_triggered_them() {
    let (mut app, line) = fixture();
    // Use a different component as the trigger to avoid recursively observing
    // the replacement itself. This models a synchronous editing side effect.
    #[derive(Component)]
    struct Mirror;
    app.add_observer(
        |event: On<Add, Mirror>, query: Query<&Note>, mut commands: Commands| {
            let note = *query.get(event.entity).unwrap();
            commands
                .entity(event.entity)
                .insert(Note { x: -note.x, ..note });
        },
    );
    let (entity, _) = create(&mut app, line);
    apply_edit(app.world_mut(), "mirror".into(), |commands| {
        commands.entity(entity).insert(Mirror);
    });
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, -100.0);
    assert_eq!(app.world().resource::<EditorHistory>().record.len(), 2);
    undo(&mut app);
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 100.0);
    redo(&mut app);
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, -100.0);
}

#[test]
fn queued_edits_cannot_leak_into_a_reopened_project() {
    let (mut app, line) = fixture();
    let (entity, _) = create(&mut app, line);
    let mut state = SystemState::<Edits>::new(app.world_mut());
    state
        .get_mut(app.world_mut())
        .once("stale edit", move |commands| {
            commands.entity(entity).insert(Note { x: 500.0, ..note() });
        });
    close_document(app.world_mut());
    app.world_mut().entity_mut(line).despawn();
    let line = app.world_mut().spawn(Line::default()).id();
    let fresh = app.world_mut().spawn((note(), ChildOf(line))).id();
    open_document(app.world_mut());
    state.apply(app.world_mut());
    assert_eq!(app.world().get::<Note>(fresh), Some(&note()));
    assert!(app.world().resource::<EditorHistory>().is_saved());
    assert_eq!(app.world().resource::<EditorHistory>().record.len(), 0);
}

#[test]
#[should_panic(expected = "document mutation outside Edits::once")]
fn authored_data_cannot_change_outside_an_edit() {
    let (mut app, line) = fixture();
    let (entity, _) = create(&mut app, line);
    app.world_mut().entity_mut(entity).insert(Note {
        above: false,
        ..note()
    });
}

#[test]
#[should_panic(expected = "object IDs cannot change")]
fn replacing_an_objects_identity_is_a_programming_error() {
    let (mut app, line) = fixture();
    let (entity, _) = create(&mut app, line);
    apply_edit(
        app.world_mut(),
        "invalid identity change".into(),
        |commands| {
            commands.entity(entity).insert(NoteId::default());
        },
    );
}
