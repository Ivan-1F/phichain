use super::create_event::CreateEventPlugin;
use super::create_note::CreateNotePlugin;
use super::history::{open_document, EditorHistory, HistoryPlugin};
use crate::hotkey::{HotkeyRegistry, HotkeyState};
use crate::id_index::IdIndexPlugin;
use crate::misc::WorkingDirectory;
use crate::selection::SelectedLine;
use crate::tab::timeline::TimelineViewport;
use crate::timeline::settings::TimelineSettings;
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

fn assert_escape_cancels(window: Option<Window>, placement_key: Option<KeyCode>, pending: bool) {
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
        ));
    let defaults = app.world().resource::<HotkeyRegistry>().0.clone();
    for (id, hotkey) in defaults {
        app.world_mut()
            .resource_mut::<HotkeyState>()
            .set(id, hotkey);
    }
    let line = app.world_mut().spawn(Line::default()).id();
    app.insert_resource(SelectedLine(line));
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
        app.world_mut().spawn((note, Pending, ChildOf(line)));
        app.world_mut().spawn((event, Pending, EventOf(line)));
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
