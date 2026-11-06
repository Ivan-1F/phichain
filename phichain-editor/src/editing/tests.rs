use super::create_event::CreateEventPlugin;
use super::create_note::CreateNotePlugin;
use super::history::{open_document, EditorHistory, HistoryPlugin};
use crate::hotkey::{HotkeyRegistry, HotkeyState};
use crate::id_index::IdIndexPlugin;
use crate::misc::WorkingDirectory;
use crate::selection::SelectedLine;
use crate::tab::timeline::TimelineViewport;
use crate::timeline::container::{
    layout_timelines, ordered_timelines, push_timeline, spawn_timeline_container,
};
use crate::timeline::event::EventTimeline;
use crate::timeline::note::NoteTimeline;
use crate::timeline::pending::{PendingOf, PendingPlugin};
use crate::timeline::settings::TimelineSettings;
use crate::timeline::target::TimelineOf;
use crate::timing::ChartTime;
use bevy::prelude::*;
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
use phichain_chart::line::Line;
use phichain_chart::note::{Note, NoteKind};
use phichain_game::audio::AudioDuration;
use phichain_game::event::EventOf;
use phichain_game::Pending;

fn editing_test_app() -> (App, Entity) {
    let mut app = App::new();
    app.init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<HotkeyRegistry>()
        .init_resource::<HotkeyState>()
        .init_resource::<TimelineSettings>()
        .init_resource::<BpmList>()
        .insert_resource(WorkingDirectory(std::path::PathBuf::new()))
        .insert_resource(ChartTime(0.0))
        .insert_resource(AudioDuration(std::time::Duration::from_secs(60)))
        .insert_resource(TimelineViewport(Rect::from_corners(
            Vec2::ZERO,
            Vec2::new(900.0, 600.0),
        )))
        .add_plugins((
            IdIndexPlugin,
            phichain_game::line::LinePlugin,
            HistoryPlugin,
            CreateNotePlugin,
            CreateEventPlugin,
            PendingPlugin,
        ));
    let defaults = app.world().resource::<HotkeyRegistry>().0.clone();
    for (id, hotkey) in defaults {
        app.world_mut()
            .resource_mut::<HotkeyState>()
            .set(id, hotkey);
    }
    spawn_timeline_container(app.world_mut());
    layout_timelines(app.world_mut());
    let line = app.world_mut().spawn(Line::default()).id();
    app.insert_resource(SelectedLine(line));
    (app, line)
}

fn assert_escape_cancels(window: Option<Window>, placement_key: Option<KeyCode>, pending: bool) {
    let (mut app, line) = editing_test_app();
    let note = Note::new(
        NoteKind::Hold {
            hold_beat: Beat::ONE,
        },
        true,
        Beat::ZERO,
        0.0,
        1.0,
    );
    let event = LineEvent {
        kind: LineEventKind::X,
        value: LineEventValue::constant(0.0),
        start_beat: Beat::ZERO,
        end_beat: Beat::ONE,
    };
    let saved_note = app.world_mut().spawn((note, ChildOf(line))).id();
    let saved_event = app.world_mut().spawn((event, EventOf(line))).id();
    if pending {
        let timelines = ordered_timelines(app.world_mut());
        app.world_mut()
            .spawn((note, Pending, PendingOf(timelines[0]), ChildOf(line)));
        app.world_mut()
            .spawn((event, Pending, PendingOf(timelines[1]), EventOf(line)));
    }
    if let Some(window) = window {
        app.world_mut().spawn(window);
    }
    open_document(app.world_mut());
    let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keyboard.press(KeyCode::Escape);
    if let Some(key) = placement_key {
        keyboard.press(key);
    }

    app.update();

    let world = app.world_mut();
    assert_eq!(
        world
            .query_filtered::<Entity, With<Pending>>()
            .iter(world)
            .count(),
        0
    );
    assert_eq!(world.query::<&Note>().iter(world).count(), 1);
    assert_eq!(world.query::<&LineEvent>().iter(world).count(), 1);
    assert_eq!(world.get::<Note>(saved_note), Some(&note));
    assert_eq!(world.get::<LineEvent>(saved_event), Some(&event));
    assert_eq!(world.resource::<EditorHistory>().entries().count(), 0);
}

fn window_at(position: Vec2) -> Window {
    let mut window = Window::default();
    window.set_cursor_position(Some(position));
    window
}

fn place_event_with_keys(first_key: KeyCode, second_key: KeyCode) -> LineEvent {
    let (mut app, line) = editing_test_app();
    app.world_mut().spawn(window_at(Vec2::new(630.0, 300.0)));
    for (start_beat, end_beat, value) in [
        (Beat::ZERO, Beat::from(0.5), 10.0),
        (Beat::ONE, Beat::from(2.0), 30.0),
    ] {
        app.world_mut().spawn((
            LineEvent {
                kind: LineEventKind::X,
                value: LineEventValue::constant(value),
                start_beat,
                end_beat,
            },
            EventOf(line),
        ));
    }
    open_document(app.world_mut());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(first_key);
    app.update();

    let world = app.world_mut();
    let pending = *world
        .query_filtered::<&LineEvent, With<Pending>>()
        .single(world)
        .expect("first key should create a pending event");
    match first_key {
        KeyCode::KeyQ => assert!(pending.value.is_constant()),
        KeyCode::KeyR => assert!(pending.value.is_transition()),
        _ => unreachable!(),
    }

    let mut keyboard = world.resource_mut::<ButtonInput<KeyCode>>();
    keyboard.reset_all();
    keyboard.press(second_key);
    app.update();

    let world = app.world_mut();
    assert_eq!(
        world
            .query_filtered::<Entity, With<Pending>>()
            .iter(world)
            .count(),
        0
    );
    let mut events = world.query::<&LineEvent>();
    let mut matching_events = events
        .iter(world)
        .filter(|event| event.start_beat == Beat::from(0.5));
    let event = *matching_events.next().expect("created event should exist");
    assert_eq!(matching_events.count(), 0);
    assert_eq!(event.kind, LineEventKind::X);
    assert_eq!(event.end_beat, Beat::from(0.75));
    assert_eq!(world.resource::<EditorHistory>().entries().count(), 1);
    event
}

#[test]
fn second_placement_key_determines_final_event_type() {
    for (first_key, second_key) in [
        (KeyCode::KeyR, KeyCode::KeyQ),
        (KeyCode::KeyQ, KeyCode::KeyR),
    ] {
        let switched = place_event_with_keys(first_key, second_key);
        let direct = place_event_with_keys(second_key, second_key);
        assert_eq!(
            switched, direct,
            "{first_key:?} followed by {second_key:?} should match direct placement"
        );
        match second_key {
            KeyCode::KeyQ => assert!(switched.value.is_constant()),
            KeyCode::KeyR => assert!(switched.value.is_transition()),
            _ => unreachable!(),
        }
    }
}

#[test]
fn escape_cancels_pending_hold_and_event_on_their_timelines() {
    for x in [300.0, 750.0] {
        assert_escape_cancels(Some(window_at(Vec2::new(x, 300.0))), None, true);
    }
}

#[test]
fn escape_takes_priority_over_confirmation_and_new_placement() {
    for x in [300.0, 750.0] {
        for key in [KeyCode::KeyQ, KeyCode::KeyR] {
            for pending in [false, true] {
                assert_escape_cancels(Some(window_at(Vec2::new(x, 300.0))), Some(key), pending);
            }
        }
    }
}

#[test]
fn escape_cancels_outside_the_timeline_or_without_a_cursor_or_window() {
    for window in [
        Some(window_at(Vec2::new(950.0, 300.0))),
        Some(Window::default()),
        None,
    ] {
        assert_escape_cancels(window, None, true);
    }
}

#[test]
fn pending_hold_follows_the_cursor_only_on_its_owner_timeline() {
    let (mut app, _) = editing_test_app();
    // note 0..400, event 400..600, note 600..900
    push_timeline(app.world_mut(), NoteTimeline, None);
    layout_timelines(app.world_mut());
    let owner = ordered_timelines(app.world_mut())[0];
    let window = app
        .world_mut()
        .spawn(window_at(Vec2::new(200.0, 300.0)))
        .id();
    open_document(app.world_mut());

    let move_cursor_and_update = |app: &mut App, position: Vec2| {
        let world = app.world_mut();
        world
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(position));
        world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.update();
        let world = app.world_mut();
        let (note, pending_of) = world
            .query_filtered::<(&Note, &PendingOf), With<Pending>>()
            .single(world)
            .unwrap();
        assert_eq!(pending_of.0, owner);
        *note
    };

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyR);
    app.update();
    let placed = move_cursor_and_update(&mut app, Vec2::new(200.0, 300.0));
    let extended = move_cursor_and_update(&mut app, Vec2::new(200.0, 100.0));
    assert!(extended.hold_beat().unwrap() > placed.hold_beat().unwrap());

    let frozen = move_cursor_and_update(&mut app, Vec2::new(750.0, 50.0));
    assert_eq!(frozen, extended);
}

fn pending_owners(world: &mut World) -> Vec<Entity> {
    world
        .query_filtered::<&PendingOf, With<Pending>>()
        .iter(world)
        .map(|owner| owner.0)
        .collect()
}

#[test]
fn starting_a_pending_object_replaces_the_existing_one() {
    let (mut app, _) = editing_test_app();
    let timelines = ordered_timelines(app.world_mut());
    let window = app
        .world_mut()
        .spawn(window_at(Vec2::new(200.0, 300.0)))
        .id();
    open_document(app.world_mut());

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyR);
    app.update();
    assert_eq!(pending_owners(app.world_mut()), vec![timelines[0]]);

    let world = app.world_mut();
    world
        .get_mut::<Window>(window)
        .unwrap()
        .set_cursor_position(Some(Vec2::new(750.0, 300.0)));
    let mut keyboard = world.resource_mut::<ButtonInput<KeyCode>>();
    keyboard.reset_all();
    keyboard.press(KeyCode::KeyQ);
    app.update();

    let world = app.world_mut();
    assert_eq!(pending_owners(world), vec![timelines[1]]);
    assert_eq!(
        world
            .query_filtered::<(), (With<Note>, With<Pending>)>()
            .iter(world)
            .count(),
        0
    );
    assert_eq!(world.resource::<EditorHistory>().entries().count(), 0);
}

#[test]
fn switching_the_selected_line_cancels_pending_objects_of_following_timelines() {
    let (mut app, line) = editing_test_app();
    let follow = ordered_timelines(app.world_mut())[0];
    let fixed = app.world_mut().spawn(TimelineOf(line)).id();
    let other_line = app.world_mut().spawn(Line::default()).id();
    let note = Note::new(NoteKind::Tap, true, Beat::ZERO, 0.0, 1.0);
    app.world_mut()
        .spawn((note, Pending, PendingOf(follow), ChildOf(line)));
    app.update();
    assert_eq!(pending_owners(app.world_mut()), vec![follow]);

    app.world_mut().resource_mut::<SelectedLine>().0 = other_line;
    app.update();
    assert!(pending_owners(app.world_mut()).is_empty());

    app.world_mut()
        .spawn((note, Pending, PendingOf(fixed), ChildOf(line)));
    app.world_mut().resource_mut::<SelectedLine>().0 = line;
    app.update();
    assert_eq!(pending_owners(app.world_mut()), vec![fixed]);
}

fn press_at(app: &mut App, window: Entity, position: Vec2, key: KeyCode) {
    let world = app.world_mut();
    world
        .get_mut::<Window>(window)
        .unwrap()
        .set_cursor_position(Some(position));
    let mut keyboard = world.resource_mut::<ButtonInput<KeyCode>>();
    keyboard.reset_all();
    keyboard.press(key);
    app.update();
}

#[test]
fn placing_notes_on_another_timeline_cancels_the_pending_hold() {
    let (mut app, _) = editing_test_app();
    // note 0..400, event 400..600, note 600..900
    push_timeline(app.world_mut(), NoteTimeline, None);
    layout_timelines(app.world_mut());
    let timelines = ordered_timelines(app.world_mut());
    let window = app.world_mut().spawn(window_at(Vec2::ZERO)).id();
    open_document(app.world_mut());

    press_at(&mut app, window, Vec2::new(200.0, 300.0), KeyCode::KeyR);
    press_at(&mut app, window, Vec2::new(750.0, 300.0), KeyCode::KeyR);
    let world = app.world_mut();
    assert_eq!(pending_owners(world), vec![timelines[2]]);
    assert_eq!(world.resource::<EditorHistory>().entries().count(), 0);

    press_at(&mut app, window, Vec2::new(200.0, 300.0), KeyCode::KeyQ);
    let world = app.world_mut();
    assert!(pending_owners(world).is_empty());
    let notes: Vec<_> = world
        .query::<&Note>()
        .iter(world)
        .map(|note| note.kind)
        .collect();
    assert_eq!(notes, vec![NoteKind::Tap]);
}

#[test]
fn placing_events_on_another_timeline_cancels_the_pending_event() {
    let (mut app, _) = editing_test_app();
    // note 0..400, event 400..600, event 600..900
    push_timeline(app.world_mut(), EventTimeline, None);
    layout_timelines(app.world_mut());
    let timelines = ordered_timelines(app.world_mut());
    let window = app.world_mut().spawn(window_at(Vec2::ZERO)).id();
    open_document(app.world_mut());

    press_at(&mut app, window, Vec2::new(500.0, 300.0), KeyCode::KeyR);
    press_at(&mut app, window, Vec2::new(750.0, 300.0), KeyCode::KeyQ);
    let world = app.world_mut();
    assert_eq!(pending_owners(world), vec![timelines[2]]);
    assert_eq!(world.query::<&LineEvent>().iter(world).count(), 1);
    assert_eq!(world.resource::<EditorHistory>().entries().count(), 0);
}
