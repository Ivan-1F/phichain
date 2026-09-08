use super::*;
use crate::id_index::IdIndexPlugin;
use crate::selection::{Selected, SelectedLine};
use bevy::ecs::system::RunSystemOnce;
use bevy::ecs::system::SystemState;
use phichain_chart::beat::Beat;
use phichain_chart::line::Line;
use phichain_chart::note::NoteKind;
use phichain_game::{Derived, Pending};

fn fixture() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((
        IdIndexPlugin,
        phichain_game::line::LinePlugin,
        HistoryPlugin,
    ));
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
    app.add_plugins((IdIndexPlugin, HistoryPlugin));
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
    app.add_systems(Update, spawn_hold_component_system);
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
fn curve_endpoints_delete_their_tracks() {
    use phichain_game::curve_note_track::{CurveNoteTrackFrom, CurveNoteTrackTo};
    let (mut app, line) = fixture();
    close_document(app.world_mut());
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
    assert!(app.world().get_entity(to).is_ok());
    for entity in [from, unrelated, track] {
        assert!(app.world().get_entity(entity).is_err());
    }
    assert_eq!(app.world().resource::<EditorHistory>().record.len(), 1);
    undo(&mut app);
    assert_eq!(
        app.world_mut().query::<&Note>().iter(app.world()).count(),
        3
    );
    let (from, to_link) = app
        .world_mut()
        .query::<(&CurveNoteTrackFrom, &CurveNoteTrackTo)>()
        .single(app.world())
        .unwrap();
    assert!(app.world().get::<Note>(from.0).is_some());
    assert_eq!(to_link.0, to);
    redo(&mut app);
    assert_eq!(
        app.world_mut().query::<&Note>().iter(app.world()).count(),
        1
    );
}

#[test]
fn deleting_curve_previews_only_records_selected_document_objects() {
    use phichain_game::curve_note_track::CurveNoteTrackFrom;

    for select_note in [false, true] {
        let (mut app, line) = fixture();
        let (origin, id) = create(&mut app, line);
        let preview = app
            .world_mut()
            .spawn((CurveNoteTrackFrom(origin), ChildOf(line), Pending, Selected))
            .id();
        if select_note {
            app.world_mut().entity_mut(origin).insert(Selected);
        }

        app.world_mut()
            .run_system_once::<_, Result, _>(
                crate::editing::delete_selected::delete_selected_system,
            )
            .unwrap()
            .unwrap();

        assert!(app.world().get_entity(preview).is_err());
        assert_eq!(app.world().get_entity(origin).is_err(), select_note);
        assert_eq!(
            app.world().resource::<EditorHistory>().record.len(),
            if select_note { 2 } else { 1 }
        );

        undo(&mut app);
        if select_note {
            let restored = entity_with_id(&mut app, id);
            assert_eq!(app.world().get::<Note>(restored), Some(&note()));
        } else {
            assert!(app.world().get_entity(origin).is_err());
        }
        assert_eq!(
            app.world_mut()
                .query::<&CurveNoteTrackFrom>()
                .iter(app.world())
                .count(),
            0
        );

        redo(&mut app);
        assert_eq!(
            app.world_mut().query::<&Note>().iter(app.world()).count(),
            if select_note { 0 } else { 1 }
        );
        assert_eq!(
            app.world_mut()
                .query::<&CurveNoteTrackFrom>()
                .iter(app.world())
                .count(),
            0
        );
    }
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
#[should_panic(expected = "document mutation outside Edits")]
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

#[test]
fn deleting_a_line_restores_its_events_tracks_and_sibling_order() {
    use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
    use phichain_chart::id::{EventId, LineId};
    use phichain_chart::serialization::SerializedLine;
    use phichain_game::curve_note_track::{CurveNoteTrackFrom, CurveNoteTrackTo};
    use phichain_game::event::EventOf;
    use phichain_game::serialization::{SerializeLine, SerializeLineParam};
    let (mut app, root) = fixture();
    let root_id = *app.world().get::<LineId>(root).unwrap();
    apply_edit(app.world_mut(), "add children".into(), |commands| {
        for name in ["first", "second", "third"] {
            let child = commands
                .spawn((Line { name: name.into() }, ChildOf(root)))
                .id();
            let from = commands.spawn((note(), ChildOf(child))).id();
            let to = commands
                .spawn((
                    Note {
                        beat: Beat::from(4.0),
                        ..note()
                    },
                    ChildOf(child),
                ))
                .id();
            commands.spawn((
                CurveNoteTrackFrom(from),
                CurveNoteTrackTo(to),
                ChildOf(child),
            ));
            commands.spawn((
                LineEvent {
                    kind: LineEventKind::X,
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                    value: LineEventValue::constant(12.0),
                },
                EventOf(child),
            ));
        }
    });
    let snapshot = |app: &mut App| {
        let root = app
            .world()
            .resource::<crate::id_index::IdIndex>()
            .entity(root_id.uuid())
            .unwrap();
        let mut line = app
            .world_mut()
            .run_system_once(move |params: SerializeLineParam| {
                SerializedLine::serialize_line(&params, root)
            })
            .unwrap();
        // Object collection order is incidental; child line order is document data.
        fn sort_objects(line: &mut SerializedLine) {
            line.notes.sort_by_key(|note| note.id);
            line.events.sort_by_key(|event| event.id);
            line.curve_note_tracks.sort_by_key(|track| track.id);
            for child in &mut line.children {
                sort_objects(child);
            }
        }
        sort_objects(&mut line);
        serde_json::to_value(line).unwrap()
    };
    let before = snapshot(&mut app);
    let second = app
        .world_mut()
        .query::<(Entity, &Line)>()
        .iter(app.world())
        .find(|(_, line)| line.name == "second")
        .unwrap()
        .0;
    apply_edit(app.world_mut(), "delete child".into(), |commands| {
        commands.entity(second).despawn();
    });
    assert_eq!(
        app.world_mut()
            .query::<&EventId>()
            .iter(app.world())
            .count(),
        2
    );
    for _ in 0..3 {
        undo(&mut app);
        assert_eq!(snapshot(&mut app), before);
        redo(&mut app);
        assert_eq!(
            app.world_mut()
                .query::<&EventId>()
                .iter(app.world())
                .count(),
            2
        );
    }
}

#[test]
fn undoing_an_inserted_parent_keeps_the_original_line_and_events() {
    use phichain_chart::serialization::SerializedLine;
    use phichain_game::event::Events;
    let (mut app, root) = fixture();
    let root_id = *app.world().get::<phichain_chart::id::LineId>(root).unwrap();
    apply_edit(app.world_mut(), "insert parent".into(), |commands| {
        let parent = crate::editing::line::spawn_line(SerializedLine::default(), commands, None);
        commands.entity(root).insert(ChildOf(parent));
    });
    let parent = app.world().get::<ChildOf>(root).unwrap().parent();
    let parent_id = *app
        .world()
        .get::<phichain_chart::id::LineId>(parent)
        .unwrap();
    let event_count = app.world().get::<Events>(parent).unwrap().len();
    undo(&mut app);
    assert!(app.world().get::<Line>(root).is_some());
    assert!(app.world().get::<ChildOf>(root).is_none());
    assert!(app.world().get_entity(parent).is_err());
    redo(&mut app);
    let index = app.world().resource::<crate::id_index::IdIndex>();
    let restored = index.entity(parent_id.uuid()).unwrap();
    assert_eq!(index.entity(root_id.uuid()), Some(root));
    assert_eq!(app.world().get::<ChildOf>(root).unwrap().parent(), restored);
    assert_eq!(
        app.world().get::<Events>(restored).unwrap().len(),
        event_count
    );
}

#[test]
fn bpm_history_restores_source_points_and_recomputes_timing() {
    use phichain_chart::bpm_list::{BpmList, BpmPoint};
    use phichain_chart::id::BpmPointId;
    let mut app = App::new();
    app.add_plugins((
        IdIndexPlugin,
        HistoryPlugin,
        crate::editing::bpm::BpmEditingPlugin,
    ));
    app.insert_resource(BpmList::default());
    crate::editing::bpm::load(app.world_mut());
    open_document(app.world_mut());
    let id = BpmPointId::new();
    apply_edit(app.world_mut(), "add bpm".into(), |commands| {
        commands.spawn((id, BpmPoint::new(Beat::from(4.0), 240.0)));
    });
    assert_eq!(
        app.world().resource::<BpmList>().time_at(Beat::from(8.0)),
        3.0
    );
    undo(&mut app);
    assert_eq!(
        app.world().resource::<BpmList>().time_at(Beat::from(8.0)),
        4.0
    );
    redo(&mut app);
    let entity = app
        .world()
        .resource::<crate::id_index::IdIndex>()
        .entity(id.uuid())
        .unwrap();
    apply_edit(app.world_mut(), "delete bpm".into(), |commands| {
        commands.entity(entity).despawn();
    });
    assert_eq!(
        app.world().resource::<BpmList>().time_at(Beat::from(8.0)),
        4.0
    );
    undo(&mut app);
    let restored = app
        .world()
        .resource::<crate::id_index::IdIndex>()
        .entity(id.uuid())
        .unwrap();
    assert_ne!(entity, restored);
    assert_eq!(
        app.world().resource::<BpmList>().time_at(Beat::from(8.0)),
        3.0
    );
    assert_eq!(
        serde_json::to_value(app.world().resource::<BpmList>()).unwrap(),
        serde_json::json!([
            {"beat": [0, 0, 1], "bpm": 120.0}, {"beat": [4, 0, 1], "bpm": 240.0}
        ])
    );
    close_document(app.world_mut());
    app.world_mut().remove_resource::<BpmList>();
    crate::editing::bpm::unload(app.world_mut());
    app.world_mut().flush();
    assert!(!app.world().contains_resource::<BpmList>());
}

#[test]
fn previews_and_generated_objects_are_excluded_from_saving() {
    use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
    use phichain_chart::serialization::SerializedLine;
    use phichain_game::event::EventOf;
    use phichain_game::serialization::{SerializeLine, SerializeLineParam};
    let (mut app, line) = fixture();
    app.world_mut().spawn((note(), Pending, ChildOf(line)));
    app.world_mut().spawn((note(), Derived, ChildOf(line)));
    app.world_mut().spawn((
        LineEvent {
            kind: LineEventKind::X,
            start_beat: Beat::ZERO,
            end_beat: Beat::ONE,
            value: LineEventValue::constant(1.0),
        },
        Pending,
        EventOf(line),
    ));
    let serialized = app
        .world_mut()
        .run_system_once(move |params: SerializeLineParam| {
            SerializedLine::serialize_line(&params, line)
        })
        .unwrap();
    assert!(serialized.notes.is_empty());
    assert!(serialized.events.is_empty());
    assert!(app.world().resource::<EditorHistory>().is_saved());
}

fn begin_gesture(app: &mut App) -> GestureId {
    app.world_mut()
        .run_system_once(|mut edits: Edits| edits.begin_gesture("drag"))
        .unwrap()
}

fn drag_x(app: &mut App, id: GestureId, entity: Entity, x: f32) {
    app.world_mut()
        .run_system_once(move |mut edits: Edits| {
            edits.gesture(id, move |commands| {
                commands.entity(entity).insert(Note { x, ..note() });
            });
        })
        .unwrap();
}

fn finish_gesture(app: &mut App, id: GestureId) {
    app.world_mut()
        .run_system_once(move |mut edits: Edits| edits.finish_gesture(id))
        .unwrap();
}

#[test]
fn gesture_updates_are_live_and_commit_as_one_edit() {
    let (mut app, line) = fixture();
    let (entity, _) = create(&mut app, line);
    app.world_mut().resource_mut::<EditorHistory>().set_saved();
    let before = app.world().resource::<EditorHistory>().head();
    let id = begin_gesture(&mut app);
    for x in [110.0, 200.0, 80.0, 160.0] {
        drag_x(&mut app, id, entity, x);
        assert_eq!(app.world().get::<Note>(entity).unwrap().x, x);
        let history = app.world().resource::<EditorHistory>();
        assert_eq!(history.record.len(), 1);
        assert_eq!(history.head(), before);
        assert!(!history.is_saved());
    }
    finish_gesture(&mut app, id);
    assert_eq!(app.world().resource::<EditorHistory>().record.len(), 2);
    undo(&mut app);
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 100.0);
    assert!(app.world().resource::<EditorHistory>().is_saved());
    redo(&mut app);
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 160.0);
}

#[test]
fn cancelled_and_no_op_gestures_preserve_saved_state_and_redo() {
    let (mut app, line) = fixture();
    let (entity, _) = create(&mut app, line);
    app.world_mut().resource_mut::<EditorHistory>().set_saved();
    apply_edit(app.world_mut(), "edit".into(), |commands| {
        commands.entity(entity).insert(Note { x: 200.0, ..note() });
    });
    undo(&mut app);
    let before = app.world().resource::<EditorHistory>().head();
    let id = begin_gesture(&mut app);
    drag_x(&mut app, id, entity, 300.0);
    app.world_mut()
        .run_system_once(move |mut edits: Edits| edits.cancel_gesture(id))
        .unwrap();
    drag_x(&mut app, id, entity, 400.0);
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 100.0);
    let id = begin_gesture(&mut app);
    drag_x(&mut app, id, entity, 150.0);
    drag_x(&mut app, id, entity, 100.0);
    finish_gesture(&mut app, id);
    let history = app.world().resource::<EditorHistory>();
    assert_eq!(history.head(), before);
    assert!(history.is_saved());
    assert!(history.record.can_redo());
    redo(&mut app);
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 200.0);
}

#[test]
fn once_finishes_the_gesture_before_deleting_and_late_updates_are_ignored() {
    let (mut app, line) = fixture();
    let (entity, note_id) = create(&mut app, line);
    let id = begin_gesture(&mut app);
    drag_x(&mut app, id, entity, 250.0);
    app.world_mut()
        .run_system_once(move |mut edits: Edits| {
            edits.once("delete", move |commands| {
                commands.entity(entity).despawn();
            });
        })
        .unwrap();
    drag_x(&mut app, id, entity, 500.0);
    assert!(app.world().get_entity(entity).is_err());
    undo(&mut app);
    let restored = entity_with_id(&mut app, note_id);
    assert_eq!(app.world().get::<Note>(restored).unwrap().x, 250.0);
    undo(&mut app);
    assert_eq!(app.world().get::<Note>(restored).unwrap().x, 100.0);
}

#[test]
fn undo_cancels_the_live_gesture_without_undoing_an_earlier_edit() {
    let (mut app, line) = fixture();
    let (entity, _) = create(&mut app, line);
    let id = begin_gesture(&mut app);
    drag_x(&mut app, id, entity, 250.0);
    undo(&mut app);
    drag_x(&mut app, id, entity, 500.0);
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 100.0);
    assert_eq!(app.world().resource::<EditorHistory>().record.head(), 1);
    undo(&mut app);
    assert!(app.world().get_entity(entity).is_err());
}

#[test]
fn gesture_ends_when_its_owner_disappears_but_not_while_idle_and_present() {
    let (mut app, line) = fixture();
    let (entity, _) = create(&mut app, line);
    let id = begin_gesture(&mut app);
    drag_x(&mut app, id, entity, 250.0);
    app.update();
    for _ in 0..3 {
        app.world_mut()
            .run_system_once(move |mut edits: Edits| edits.keep_gesture_alive(id))
            .unwrap();
        app.update();
        assert!(app.world().resource::<EditorHistory>().has_gesture());
    }
    app.update();
    assert!(!app.world().resource::<EditorHistory>().has_gesture());
    assert_eq!(app.world().resource::<EditorHistory>().record.len(), 2);
    undo(&mut app);
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 100.0);
}

#[test]
fn saving_a_gesture_creates_a_stable_saved_revision() {
    let (mut app, line) = fixture();
    let (entity, _) = create(&mut app, line);
    let id = begin_gesture(&mut app);
    drag_x(&mut app, id, entity, 250.0);
    app.world_mut()
        .resource_scope(|world, mut history: Mut<EditorHistory>| {
            history.finish_gesture(world);
            history.set_saved();
        });
    drag_x(&mut app, id, entity, 500.0);
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 250.0);
    undo(&mut app);
    assert!(!app.world().resource::<EditorHistory>().is_saved());
    redo(&mut app);
    assert!(app.world().resource::<EditorHistory>().is_saved());
}

#[test]
#[should_panic(expected = "document mutation outside Edits")]
fn an_open_gesture_does_not_allow_unscoped_mutations() {
    let (mut app, line) = fixture();
    let (entity, _) = create(&mut app, line);
    begin_gesture(&mut app);
    app.world_mut()
        .entity_mut(entity)
        .insert(Note { x: 99.0, ..note() });
}
