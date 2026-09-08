use super::*;
use crate::editing::history::{open_document, HistoryPlugin};
use crate::id_index::IdIndexPlugin;
use bevy::ecs::system::SystemState;
use egui::{Context, Modifiers, PointerButton, Pos2, RawInput, Rect};
use phichain_chart::beat::Beat;
use phichain_chart::note::{Note, NoteKind};

fn fixture() -> (App, Context, Entity) {
    let mut app = App::new();
    app.add_plugins((IdIndexPlugin, HistoryPlugin));
    let entity = app
        .world_mut()
        .spawn(Note::new(NoteKind::Tap, true, Beat::ONE, 100.0, 1.0))
        .id();
    open_document(app.world_mut());
    (app, Context::default(), entity)
}

fn frame(app: &mut App, ctx: &Context, entity: Entity, events: Vec<Event>, visible: bool) -> Rect {
    let mut rect = Rect::NOTHING;
    let _ = ctx.run(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(400.0, 200.0))),
            events,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                if visible {
                    let note = *app.world().get::<Note>(entity).unwrap();
                    let mut state: SystemState<Edits> = SystemState::new(app.world_mut());
                    let mut edits = state.get_mut(app.world_mut());
                    rect = edits
                        .component(entity, &note, "edit x")
                        .field(ui, "x", "X", |ui, note| {
                            ui.add(egui::DragValue::new(&mut note.x).speed(1))
                        })
                        .rect;
                    state.apply(app.world_mut());
                }
            });
        },
    );
    app.update();
    rect
}

fn pointer(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
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

fn focus(app: &mut App, ctx: &Context, entity: Entity) {
    let pos = frame(app, ctx, entity, vec![], true).center();
    frame(
        app,
        ctx,
        entity,
        vec![Event::PointerMoved(pos), pointer(pos, true)],
        true,
    );
    frame(app, ctx, entity, vec![pointer(pos, false)], true);
    frame(app, ctx, entity, vec![], true);
}

#[test]
fn typed_values_preview_live_enter_commits_and_escape_restores() {
    let (mut app, ctx, entity) = fixture();
    focus(&mut app, &ctx, entity);
    frame(
        &mut app,
        &ctx,
        entity,
        vec![Event::Text("200".into())],
        true,
    );
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 200.0);
    assert!(app.world().resource::<EditorHistory>().has_gesture());
    frame(&mut app, &ctx, entity, vec![Event::Text("5".into())], true);
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 2005.0);
    frame(&mut app, &ctx, entity, vec![key(Key::Enter)], true);
    assert!(!app.world().resource::<EditorHistory>().has_gesture());
    app.world_mut()
        .resource_scope(|world, mut history: Mut<EditorHistory>| history.undo(world))
        .unwrap();
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 100.0);
    focus(&mut app, &ctx, entity);
    frame(
        &mut app,
        &ctx,
        entity,
        vec![Event::Text("300".into())],
        true,
    );
    frame(&mut app, &ctx, entity, vec![key(Key::Escape)], true);
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 100.0);
    assert!(app.world().resource::<EditorHistory>().is_saved());
    app.world_mut()
        .resource_scope(|world, mut history: Mut<EditorHistory>| history.redo(world))
        .unwrap();
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 2005.0);
}

#[test]
fn dragging_previews_and_release_outside_commits_one_edit() {
    let (mut app, ctx, entity) = fixture();
    let pos = frame(&mut app, &ctx, entity, vec![], true).center();
    frame(
        &mut app,
        &ctx,
        entity,
        vec![Event::PointerMoved(pos), pointer(pos, true)],
        true,
    );
    frame(
        &mut app,
        &ctx,
        entity,
        vec![Event::PointerMoved(pos + egui::vec2(30.0, 0.0))],
        true,
    );
    frame(
        &mut app,
        &ctx,
        entity,
        vec![Event::PointerMoved(pos + egui::vec2(50.0, 0.0))],
        true,
    );
    let value = app.world().get::<Note>(entity).unwrap().x;
    assert!(value > 100.0);
    assert!(app.world().resource::<EditorHistory>().has_gesture());
    frame(&mut app, &ctx, entity, vec![], true);
    assert!(app.world().resource::<EditorHistory>().has_gesture());
    frame(
        &mut app,
        &ctx,
        entity,
        vec![pointer(pos + egui::vec2(50.0, 0.0), false)],
        true,
    );
    assert!(!app.world().resource::<EditorHistory>().has_gesture());
    app.world_mut()
        .resource_scope(|world, mut history: Mut<EditorHistory>| history.undo(world))
        .unwrap();
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 100.0);
    app.world_mut()
        .resource_scope(|world, mut history: Mut<EditorHistory>| history.redo(world))
        .unwrap();
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, value);
}

#[test]
fn hiding_the_input_commits_the_last_visible_value() {
    let (mut app, ctx, entity) = fixture();
    focus(&mut app, &ctx, entity);
    frame(
        &mut app,
        &ctx,
        entity,
        vec![Event::Text("300".into())],
        true,
    );
    frame(&mut app, &ctx, entity, vec![], false);
    assert!(!app.world().resource::<EditorHistory>().has_gesture());
    app.world_mut()
        .resource_scope(|world, mut history: Mut<EditorHistory>| history.undo(world))
        .unwrap();
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 100.0);
}

#[test]
fn escape_during_drag_ignores_further_motion_until_release() {
    let (mut app, ctx, entity) = fixture();
    let pos = frame(&mut app, &ctx, entity, vec![], true).center();
    frame(
        &mut app,
        &ctx,
        entity,
        vec![Event::PointerMoved(pos), pointer(pos, true)],
        true,
    );
    frame(
        &mut app,
        &ctx,
        entity,
        vec![Event::PointerMoved(pos + egui::vec2(30.0, 0.0))],
        true,
    );
    assert_ne!(app.world().get::<Note>(entity).unwrap().x, 100.0);
    frame(&mut app, &ctx, entity, vec![key(Key::Escape)], true);
    frame(
        &mut app,
        &ctx,
        entity,
        vec![Event::PointerMoved(pos + egui::vec2(80.0, 0.0))],
        true,
    );
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 100.0);
    frame(
        &mut app,
        &ctx,
        entity,
        vec![pointer(pos + egui::vec2(80.0, 0.0), false)],
        true,
    );
    assert!(app.world().resource::<EditorHistory>().is_saved());
}

#[test]
fn escape_cancels_even_when_the_input_is_no_longer_drawn() {
    let (mut app, ctx, entity) = fixture();
    app.init_resource::<ButtonInput<KeyCode>>()
        .add_systems(PreUpdate, interrupt_gesture_system);
    focus(&mut app, &ctx, entity);
    frame(
        &mut app,
        &ctx,
        entity,
        vec![Event::Text("300".into())],
        true,
    );
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Escape);
    frame(&mut app, &ctx, entity, vec![], false);
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 100.0);
    assert!(app.world().resource::<EditorHistory>().is_saved());
}

use super::component::ComponentEditor;
use crate::ui::widgets::beat_value::BeatValue;

fn fields_frame(
    app: &mut App,
    ctx: &Context,
    entity: Entity,
    events: Vec<Event>,
    draw: &mut impl FnMut(&mut ComponentEditor<'_, '_, '_, Note>, &mut egui::Ui) -> Vec<Rect>,
) -> (Vec<Rect>, Vec<(String, Pos2)>) {
    let mut rects = Vec::new();
    let output = ctx.run(
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(500.0, 300.0))),
            events,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let note = *app.world().get::<Note>(entity).unwrap();
                let mut state: SystemState<Edits> = SystemState::new(app.world_mut());
                let mut edits = state.get_mut(app.world_mut());
                let mut editor = edits.component(entity, &note, "edit note");
                rects = draw(&mut editor, ui);
                state.apply(app.world_mut());
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
    (rects, texts)
}

#[test]
fn a_checkbox_keeps_the_same_frame_numeric_edit_and_has_its_own_history() {
    let (mut app, ctx, entity) = fixture();
    let minimum = std::cell::Cell::new(0.0);
    let mut draw = |editor: &mut ComponentEditor<'_, '_, '_, Note>, ui: &mut egui::Ui| {
        vec![
            editor
                .field(ui, "x", "X", |ui, note| {
                    ui.add(egui::DragValue::new(&mut note.x).range(minimum.get()..=f32::MAX))
                })
                .rect,
            editor
                .field(ui, "above", "Above", |ui, note| {
                    ui.checkbox(&mut note.above, "")
                })
                .rect,
        ]
    };
    let rects = fields_frame(&mut app, &ctx, entity, vec![], &mut draw).0;
    let x = rects[0].center();
    let above = rects[1].center();
    fields_frame(
        &mut app,
        &ctx,
        entity,
        vec![Event::PointerMoved(x), pointer(x, true)],
        &mut draw,
    );
    fields_frame(&mut app, &ctx, entity, vec![pointer(x, false)], &mut draw);
    fields_frame(&mut app, &ctx, entity, vec![], &mut draw);
    fields_frame(
        &mut app,
        &ctx,
        entity,
        vec![Event::Text("200".into())],
        &mut draw,
    );
    minimum.set(250.0);
    fields_frame(
        &mut app,
        &ctx,
        entity,
        vec![
            Event::PointerMoved(above),
            pointer(above, true),
            pointer(above, false),
        ],
        &mut draw,
    );
    let note = app.world().get::<Note>(entity).unwrap();
    assert_eq!(note.x, 250.0);
    assert!(!note.above);
    assert!(!app.world().resource::<EditorHistory>().has_gesture());
    app.world_mut()
        .resource_scope(|world, mut history: Mut<EditorHistory>| history.undo(world))
        .unwrap();
    let note = app.world().get::<Note>(entity).unwrap();
    assert_eq!(note.x, 250.0);
    assert!(note.above);
    app.world_mut()
        .resource_scope(|world, mut history: Mut<EditorHistory>| history.undo(world))
        .unwrap();
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 100.0);
    assert!(app.world().resource::<EditorHistory>().is_saved());
}

#[test]
fn beat_controls_commit_independently_and_escape_only_cancels_the_active_input() {
    let mut app = App::new();
    app.add_plugins((IdIndexPlugin, HistoryPlugin));
    let original = phichain_chart::beat!(2, 1, 4);
    let entity = app
        .world_mut()
        .spawn(Note::new(NoteKind::Tap, true, original, 100.0, 1.0))
        .id();
    open_document(app.world_mut());
    let ctx = Context::default();
    let mut draw = |editor: &mut ComponentEditor<'_, '_, '_, Note>, ui: &mut egui::Ui| {
        vec![
            editor
                .field(ui, "beat", "Beat", |ui, note| {
                    ui.add(BeatValue::new(&mut note.beat))
                })
                .rect,
        ]
    };
    let focus = |app: &mut App, text: &str, draw: &mut _| {
        let texts = fields_frame(app, &ctx, entity, vec![], draw).1;
        let pos = texts
            .iter()
            .find(|(label, _)| label == text)
            .expect("beat input")
            .1;
        fields_frame(
            app,
            &ctx,
            entity,
            vec![Event::PointerMoved(pos), pointer(pos, true)],
            draw,
        );
        fields_frame(app, &ctx, entity, vec![pointer(pos, false)], draw);
        fields_frame(app, &ctx, entity, vec![], draw);
    };
    focus(&mut app, "2", &mut draw);
    fields_frame(
        &mut app,
        &ctx,
        entity,
        vec![Event::Text("3".into())],
        &mut draw,
    );
    assert_eq!(
        app.world().get::<Note>(entity).unwrap().beat,
        phichain_chart::beat!(3, 1, 4)
    );
    focus(&mut app, "4", &mut draw);
    fields_frame(
        &mut app,
        &ctx,
        entity,
        vec![Event::Text("8".into())],
        &mut draw,
    );
    fields_frame(&mut app, &ctx, entity, vec![key(Key::Enter)], &mut draw);
    assert_eq!(
        app.world().get::<Note>(entity).unwrap().beat,
        phichain_chart::beat!(3, 1, 8)
    );
    focus(&mut app, "1", &mut draw);
    fields_frame(
        &mut app,
        &ctx,
        entity,
        vec![Event::Text("5".into())],
        &mut draw,
    );
    assert_eq!(
        app.world().get::<Note>(entity).unwrap().beat,
        phichain_chart::beat!(3, 5, 8)
    );
    fields_frame(&mut app, &ctx, entity, vec![key(Key::Escape)], &mut draw);
    assert_eq!(
        app.world().get::<Note>(entity).unwrap().beat,
        phichain_chart::beat!(3, 1, 8)
    );
    app.world_mut()
        .resource_scope(|world, mut history: Mut<EditorHistory>| history.undo(world))
        .unwrap();
    assert_eq!(
        app.world().get::<Note>(entity).unwrap().beat,
        phichain_chart::beat!(3, 1, 4)
    );
    app.world_mut()
        .resource_scope(|world, mut history: Mut<EditorHistory>| history.undo(world))
        .unwrap();
    assert_eq!(app.world().get::<Note>(entity).unwrap().beat, original);
}

#[test]
fn rendering_unchanged_fields_and_clamped_inputs_does_not_replace_components() {
    let (mut app, ctx, entity) = fixture();
    #[derive(Resource, Default)]
    struct Replacements(usize);
    app.init_resource::<Replacements>()
        .add_observer(|_: On<Replace, Note>, mut count: ResMut<Replacements>| count.0 += 1);
    let mut draw = |editor: &mut ComponentEditor<'_, '_, '_, Note>, ui: &mut egui::Ui| {
        vec![
            editor
                .field(ui, "x", "X", |ui, note| {
                    ui.add(egui::DragValue::new(&mut note.x).range(0.0..=100.0))
                })
                .rect,
        ]
    };
    let pos = fields_frame(&mut app, &ctx, entity, vec![], &mut draw).0[0].center();
    fields_frame(&mut app, &ctx, entity, vec![], &mut draw);
    fields_frame(
        &mut app,
        &ctx,
        entity,
        vec![Event::PointerMoved(pos), pointer(pos, true)],
        &mut draw,
    );
    fields_frame(
        &mut app,
        &ctx,
        entity,
        vec![Event::PointerMoved(pos + egui::vec2(40.0, 0.0))],
        &mut draw,
    );
    fields_frame(
        &mut app,
        &ctx,
        entity,
        vec![pointer(pos + egui::vec2(40.0, 0.0), false)],
        &mut draw,
    );
    let mut disabled = |editor: &mut ComponentEditor<'_, '_, '_, Note>, ui: &mut egui::Ui| {
        vec![
            ui.add_enabled_ui(false, |ui| {
                editor
                    .field(ui, "x", "X", |ui, note| {
                        ui.add(egui::DragValue::new(&mut note.x).range(0.0..=10.0))
                    })
                    .rect
            })
            .inner,
        ]
    };
    fields_frame(&mut app, &ctx, entity, vec![], &mut disabled);
    assert_eq!(app.world().get::<Note>(entity).unwrap().x, 100.0);
    assert_eq!(app.world().resource::<Replacements>().0, 0);
    assert!(app.world().resource::<EditorHistory>().is_saved());
}
