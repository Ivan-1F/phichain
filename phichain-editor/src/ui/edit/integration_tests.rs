use super::*;
use crate::editing::history::{open_document, HistoryPlugin};
use crate::id_index::IdIndexPlugin;
use bevy::ecs::system::{RunSystemOnce, SystemState};
use egui::{Context, Modifiers, PointerButton, Pos2, RawInput, Rect};
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::{BpmList, BpmPoint};
use phichain_chart::curve_note_track::CurveNoteTrackOptions;
use phichain_chart::easing::Easing;
use phichain_chart::id::CurveNoteTrackId;
use phichain_chart::offset::Offset;
use phichain_chart::project::{Project, ProjectMeta, ProjectPath};
use std::path::PathBuf;

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((IdIndexPlugin, HistoryPlugin));
    app
}

fn frame(
    app: &mut App,
    ctx: &Context,
    events: Vec<Event>,
    draw: &mut impl FnMut(&mut World, egui::Ui),
) -> Vec<(String, Pos2)> {
    let output = ctx.run(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(600.0, 800.0))),
            events,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                draw(app.world_mut(), ui.new_child(egui::UiBuilder::new()));
            });
        },
    );
    app.update();
    fn collect(shape: egui::Shape, texts: &mut Vec<(String, Pos2)>) {
        match shape {
            egui::Shape::Text(text) => texts.push((
                text.galley.text().into(),
                text.visual_bounding_rect().center(),
            )),
            egui::Shape::Vec(shapes) => shapes.into_iter().for_each(|shape| collect(shape, texts)),
            _ => {}
        }
    }
    let mut texts = Vec::new();
    for shape in output.shapes {
        collect(shape.shape, &mut texts);
    }
    texts
}

fn pointer(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        pressed,
        button: PointerButton::Primary,
        modifiers: Modifiers::NONE,
    }
}

fn key(key: Key) -> Event {
    Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

fn click(app: &mut App, ctx: &Context, pos: Pos2, draw: &mut impl FnMut(&mut World, egui::Ui)) {
    frame(
        app,
        ctx,
        vec![Event::PointerMoved(pos), pointer(pos, true)],
        draw,
    );
    frame(app, ctx, vec![pointer(pos, false)], draw);
    frame(app, ctx, vec![], draw);
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

#[test]
fn bpm_text_input_keeps_focus_when_the_point_changes_sort_order() {
    let mut app = app();
    app.add_plugins(crate::editing::bpm::BpmEditingPlugin)
        .insert_resource(BpmList::default());
    app.world_mut().spawn(BpmPoint::new(Beat::ZERO, 120.0));
    let entity = app
        .world_mut()
        .spawn(BpmPoint::new(Beat::from(4.0), 150.0))
        .id();
    app.world_mut().spawn(BpmPoint::new(Beat::from(8.0), 180.0));
    open_document(app.world_mut());
    let ctx = Context::default();
    let mut draw = |world: &mut World, ui| {
        world
            .run_system_once_with(crate::tab::bpm_list::bpm_list_tab, ui)
            .unwrap();
    };
    let texts = frame(&mut app, &ctx, vec![], &mut draw);
    let pos = texts.iter().find(|(text, _)| text == "4").unwrap().1;
    click(&mut app, &ctx, pos, &mut draw);
    frame(&mut app, &ctx, vec![Event::Text("9".into())], &mut draw);
    assert_eq!(
        app.world().get::<BpmPoint>(entity).unwrap().beat,
        Beat::from(9.0)
    );
    frame(&mut app, &ctx, vec![Event::Text("0".into())], &mut draw);
    assert_eq!(
        app.world().get::<BpmPoint>(entity).unwrap().beat,
        Beat::from(90.0)
    );
    frame(&mut app, &ctx, vec![key(Key::Enter)], &mut draw);
    assert_eq!(
        app.world().resource::<BpmList>().0.last().unwrap().beat,
        Beat::from(90.0)
    );
    undo(&mut app);
    assert_eq!(
        app.world().get::<BpmPoint>(entity).unwrap().beat,
        Beat::from(4.0)
    );
    assert_eq!(app.world().resource::<BpmList>().0[1].beat, Beat::from(4.0));
    assert!(app.world().resource::<EditorHistory>().is_saved());
}

#[test]
fn project_fields_preview_commit_and_cancel_through_the_same_component_history() {
    let mut app = app();
    app.insert_resource(Offset(0.0))
        .insert_resource(Project {
            path: ProjectPath(PathBuf::new()),
            meta: ProjectMeta {
                name: "Original".into(),
                ..Default::default()
            },
            id: uuid::Uuid::new_v4(),
        })
        .add_plugins(crate::editing::project_settings::ProjectSettingsPlugin);
    crate::editing::project_settings::load(app.world_mut());
    open_document(app.world_mut());
    let ctx = Context::default();
    let mut draw = |world: &mut World, ui: egui::Ui| {
        world
            .run_system_once_with(crate::tab::chart_basic_setting::chart_basic_setting_tab, ui)
            .unwrap();
    };
    let texts = frame(&mut app, &ctx, vec![], &mut draw);
    let pos = texts.iter().find(|(text, _)| text == "Original").unwrap().1;
    click(&mut app, &ctx, pos, &mut draw);
    let mut select_all = key(Key::A);
    if let Event::Key { modifiers, .. } = &mut select_all {
        modifiers.command = true;
    }
    frame(&mut app, &ctx, vec![select_all], &mut draw);
    frame(&mut app, &ctx, vec![Event::Text("Chart".into())], &mut draw);
    frame(
        &mut app,
        &ctx,
        vec![Event::Text(" title".into())],
        &mut draw,
    );
    assert_eq!(app.world().resource::<Project>().meta.name, "Chart title");
    frame(&mut app, &ctx, vec![key(Key::Enter)], &mut draw);
    assert!(!app.world().resource::<EditorHistory>().has_gesture());
    undo(&mut app);
    assert_eq!(app.world().resource::<Project>().meta.name, "Original");
    redo(&mut app);
    assert_eq!(app.world().resource::<Project>().meta.name, "Chart title");
    click(&mut app, &ctx, pos, &mut draw);
    frame(&mut app, &ctx, vec![Event::Text("!".into())], &mut draw);
    frame(&mut app, &ctx, vec![key(Key::Escape)], &mut draw);
    assert_eq!(app.world().resource::<Project>().meta.name, "Chart title");

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<ProjectMeta>>()
        .single(app.world())
        .unwrap();
    app.world_mut()
        .run_system_once(move |mut edits: Edits| {
            edits.once("offset", move |commands| {
                commands.entity(entity).insert(Offset(125.0));
            });
        })
        .unwrap();
    assert_eq!(*app.world().resource::<Offset>(), Offset(125.0));
    undo(&mut app);
    assert_eq!(*app.world().resource::<Offset>(), Offset(0.0));
    redo(&mut app);
    assert_eq!(*app.world().resource::<Offset>(), Offset(125.0));
    crate::editing::history::close_document(app.world_mut());
    crate::editing::project_settings::unload(app.world_mut());
    assert!(app.world().get_entity(entity).is_err());
    assert!(app.world().resource::<EditorHistory>().is_saved());
}

#[test]
fn easing_handles_follow_screen_axes_and_record_the_release_movement() {
    use crate::ui::widgets::easing::EasingGraph;
    for inverse in [false, true] {
        for mirror in [false, true] {
            let mut app = app();
            let original = Easing::Custom {
                x1: 0.25,
                y1: 0.25,
                x2: 0.75,
                y2: 0.75,
            };
            let entity = app
                .world_mut()
                .spawn((
                    CurveNoteTrackId::new(),
                    CurveNoteTrackOptions {
                        curve: original,
                        ..Default::default()
                    },
                ))
                .id();
            open_document(app.world_mut());
            let ctx = Context::default();
            let graph = Rect::from_min_size(egui::pos2(100.0, 100.0), egui::vec2(200.0, 200.0));
            let mut draw = |world: &mut World, mut ui: egui::Ui| {
                let options = world.get::<CurveNoteTrackOptions>(entity).unwrap().clone();
                let mut state: SystemState<Edits> = SystemState::new(world);
                state
                    .get_mut(world)
                    .component(entity, &options, "curve")
                    .edit(&mut ui, "graph", |ui, options| {
                        ui.add(
                            EasingGraph::new(&mut options.curve)
                                .rect(graph)
                                .inverse(inverse)
                                .mirror(mirror),
                        );
                    });
                state.apply(world);
            };
            let point = if inverse {
                egui::pos2(if mirror { 0.75 } else { 0.25 }, 0.75)
            } else {
                egui::pos2(0.25, if mirror { 0.25 } else { 0.75 })
            };
            let pos = graph.min + point.to_vec2() * graph.size();
            frame(&mut app, &ctx, vec![], &mut draw);
            frame(
                &mut app,
                &ctx,
                vec![Event::PointerMoved(pos), pointer(pos, true)],
                &mut draw,
            );
            frame(
                &mut app,
                &ctx,
                vec![Event::PointerMoved(pos + egui::vec2(20.0, 0.0))],
                &mut draw,
            );
            let release = pos + egui::vec2(40.0, 20.0);
            frame(
                &mut app,
                &ctx,
                vec![Event::PointerMoved(release), pointer(release, false)],
                &mut draw,
            );
            let curve = app
                .world()
                .get::<CurveNoteTrackOptions>(entity)
                .unwrap()
                .curve;
            let Easing::Custom { x1, y1, .. } = curve else {
                unreachable!()
            };
            let expected = if inverse {
                (0.15, if mirror { 0.05 } else { 0.45 })
            } else {
                (0.45, if mirror { 0.35 } else { 0.15 })
            };
            assert!(
                (x1 - expected.0).abs() < 0.0001,
                "{inverse} {mirror}: {curve:?}"
            );
            assert!(
                (y1 - expected.1).abs() < 0.0001,
                "{inverse} {mirror}: {curve:?}"
            );
            assert!(!app.world().resource::<EditorHistory>().has_gesture());
            undo(&mut app);
            assert_eq!(
                app.world()
                    .get::<CurveNoteTrackOptions>(entity)
                    .unwrap()
                    .curve,
                original
            );
            assert!(app.world().resource::<EditorHistory>().is_saved());
            redo(&mut app);
            assert_eq!(
                app.world()
                    .get::<CurveNoteTrackOptions>(entity)
                    .unwrap()
                    .curve,
                curve
            );
        }
    }
}

#[test]
fn easing_parameter_drag_forms_one_edit_for_steps_and_elastic() {
    use crate::ui::widgets::easing::EasingParameter;
    for original in [Easing::Steps { count: 4 }, Easing::Elastic { omega: 20.0 }] {
        let mut app = app();
        let entity = app
            .world_mut()
            .spawn((
                CurveNoteTrackId::new(),
                CurveNoteTrackOptions {
                    curve: original,
                    ..Default::default()
                },
            ))
            .id();
        open_document(app.world_mut());
        let ctx = Context::default();
        let rect = std::cell::Cell::new(Rect::NOTHING);
        let mut draw = |world: &mut World, mut ui: egui::Ui| {
            let options = world.get::<CurveNoteTrackOptions>(entity).unwrap().clone();
            let mut state: SystemState<Edits> = SystemState::new(world);
            rect.set(
                state
                    .get_mut(world)
                    .component(entity, &options, "parameter")
                    .edit(&mut ui, "parameter", |ui, options| {
                        ui.add(EasingParameter(&mut options.curve))
                    })
                    .rect,
            );
            state.apply(world);
        };
        frame(&mut app, &ctx, vec![], &mut draw);
        let pos = rect.get().center();
        frame(
            &mut app,
            &ctx,
            vec![Event::PointerMoved(pos), pointer(pos, true)],
            &mut draw,
        );
        frame(
            &mut app,
            &ctx,
            vec![Event::PointerMoved(pos + egui::vec2(10.0, 0.0))],
            &mut draw,
        );
        frame(
            &mut app,
            &ctx,
            vec![Event::PointerMoved(pos + egui::vec2(20.0, 0.0))],
            &mut draw,
        );
        frame(
            &mut app,
            &ctx,
            vec![pointer(pos + egui::vec2(20.0, 0.0), false)],
            &mut draw,
        );
        assert_ne!(
            app.world()
                .get::<CurveNoteTrackOptions>(entity)
                .unwrap()
                .curve,
            original
        );
        undo(&mut app);
        assert_eq!(
            app.world()
                .get::<CurveNoteTrackOptions>(entity)
                .unwrap()
                .curve,
            original
        );
        assert!(app.world().resource::<EditorHistory>().is_saved());
    }
}

fn timeline_resources(app: &mut App) {
    app.insert_resource(crate::timeline::settings::TimelineSettings {
        zoom: 0.5,
        ..Default::default()
    })
    .insert_resource(BpmList::default())
    .insert_resource(crate::timing::ChartTime(0.0))
    .insert_resource(crate::tab::timeline::TimelineViewport(
        bevy::math::Rect::from_corners(Vec2::ZERO, Vec2::new(600.0, 800.0)),
    ))
    .insert_resource(phichain_game::audio::AudioDuration(
        std::time::Duration::from_secs(60),
    ));
}

#[test]
fn event_inspector_switches_mode_immediately_and_keeps_separate_edits() {
    use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
    let mut app = app();
    timeline_resources(&mut app);
    app.add_plugins(crate::tab::inspector::InspectorPlugin);
    let original = LineEvent {
        kind: LineEventKind::X,
        start_beat: Beat::ONE,
        end_beat: Beat::from(3.0),
        value: LineEventValue::constant(123.0),
    };
    let entity = app
        .world_mut()
        .spawn((original, crate::selection::Selected))
        .id();
    open_document(app.world_mut());
    let ctx = Context::default();
    let mut draw = |world: &mut World, ui| {
        world
            .run_system_once_with(crate::tab::inspector::inspector_ui_system, ui)
            .unwrap();
    };
    let texts = frame(&mut app, &ctx, vec![], &mut draw);
    let value = texts.iter().find(|(text, _)| text == "123").unwrap().1;
    click(&mut app, &ctx, value, &mut draw);
    frame(&mut app, &ctx, vec![Event::Text("234".into())], &mut draw);
    let texts = frame(&mut app, &ctx, vec![], &mut draw);
    let transition = texts
        .iter()
        .find(|(text, _)| text == t!("tab.inspector.single_event.transition").as_ref())
        .unwrap()
        .1;
    frame(
        &mut app,
        &ctx,
        vec![Event::PointerMoved(transition), pointer(transition, true)],
        &mut draw,
    );
    let texts = frame(&mut app, &ctx, vec![pointer(transition, false)], &mut draw);
    assert!(texts
        .iter()
        .any(|(text, _)| text == t!("tab.inspector.single_event.end_value").as_ref()));
    assert_eq!(
        app.world().get::<LineEvent>(entity).unwrap().value,
        LineEventValue::transition(234.0, 234.0, Easing::Linear)
    );
    undo(&mut app);
    assert_eq!(
        app.world().get::<LineEvent>(entity).unwrap().value,
        LineEventValue::constant(234.0)
    );
    undo(&mut app);
    assert_eq!(*app.world().get::<LineEvent>(entity).unwrap(), original);
    assert!(app.world().resource::<EditorHistory>().is_saved());
}

#[test]
fn event_timeline_applies_handle_edits_and_undo_restores_the_range() {
    use crate::timeline::{Timeline, TimelineContext};
    use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
    let mut app = app();
    timeline_resources(&mut app);
    app.add_message::<crate::selection::Select>()
        .add_message::<crate::timing::SeekTo>();
    let line = app
        .world_mut()
        .spawn(phichain_chart::line::Line::default())
        .id();
    app.insert_resource(crate::selection::SelectedLine(line));
    let original = LineEvent {
        kind: LineEventKind::X,
        start_beat: Beat::ONE,
        end_beat: Beat::from(3.0),
        value: LineEventValue::constant(123.0),
    };
    let entity = app
        .world_mut()
        .spawn((original, phichain_game::event::EventOf(line)))
        .id();
    open_document(app.world_mut());
    let ctx = Context::default();
    let mut state: SystemState<TimelineContext> = SystemState::new(app.world_mut());
    let timeline = state.get_mut(app.world_mut());
    let y = timeline.beat_to_y(original.end_beat);
    let target_y = timeline.beat_to_y(Beat::from(4.0));
    let pos = egui::pos2(60.0, y + 2.0);
    let end = egui::pos2(60.0, target_y + 2.0);
    let mut draw = |world: &mut World, mut ui: egui::Ui| {
        crate::timeline::event::EventTimeline::new(line).ui(
            &mut ui,
            world,
            Rect::from_min_size(Pos2::ZERO, egui::vec2(600.0, 800.0)),
        );
    };
    frame(&mut app, &ctx, vec![], &mut draw);
    frame(
        &mut app,
        &ctx,
        vec![Event::PointerMoved(pos), pointer(pos, true)],
        &mut draw,
    );
    frame(&mut app, &ctx, vec![Event::PointerMoved(end)], &mut draw);
    frame(&mut app, &ctx, vec![pointer(end, false)], &mut draw);
    assert_eq!(
        app.world().get::<LineEvent>(entity).unwrap().end_beat,
        Beat::from(4.0)
    );
    assert_eq!(
        app.world().get::<LineEvent>(entity).unwrap().start_beat,
        Beat::ONE
    );
    undo(&mut app);
    assert_eq!(*app.world().get::<LineEvent>(entity).unwrap(), original);
    assert!(app.world().resource::<EditorHistory>().is_saved());
}
