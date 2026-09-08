use crate::editing::history::{EditorHistory, Edits, GestureId};
use bevy::prelude::*;
use egui::{Event, Key, Response};

/// Connect a value-changing widget to history. Only interaction identity lives
/// in egui memory; old document values are retained by the ECS recorder.
pub fn gesture_response(
    edits: &mut Edits,
    response: &Response,
    description: impl Into<String>,
    edit: impl FnOnce(&mut Commands) + Send + 'static,
) {
    let key = response.id.with("history_gesture");
    let mut gesture = response.ctx.data(|data| data.get_temp::<GestureId>(key));
    let (escape, focused) = response
        .ctx
        .input(|input| (escape_pressed(&input.raw.events), input.focused));
    if escape || !focused {
        if let Some(id) = gesture {
            if escape {
                edits.cancel_gesture(id);
            } else {
                edits.finish_gesture(id);
            }
        }
        response.surrender_focus();
    } else {
        if response.drag_started() || response.gained_focus() {
            if let Some(id) = gesture.take() {
                edits.finish_gesture(id);
            }
        }
        if gesture.is_none() && (response.changed() || response.drag_started()) {
            gesture = Some(edits.begin_gesture(description));
        }
        if let Some(id) = gesture {
            if response.changed() {
                edits.gesture(id, edit);
            }
            edits.keep_gesture_alive(id);
        }
    }
    if response.drag_stopped()
        || response.lost_focus()
        || (!response.dragged() && !response.has_focus())
    {
        if let Some(id) = gesture.take() {
            edits.finish_gesture(id);
        }
    }
    response.ctx.data_mut(|data| {
        if let Some(id) = gesture {
            data.insert_temp(key, id);
        } else {
            data.remove::<GestureId>(key);
        }
    });
}

fn escape_pressed(events: &[Event]) -> bool {
    events.iter().any(|event| {
        matches!(
            event,
            Event::Key {
                key: Key::Escape,
                pressed: true,
                ..
            }
        )
    })
}

/// Resolve cancellation before shortcuts can remove the owning widget or selection.
pub fn interrupt_gesture_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    history: Res<EditorHistory>,
    mut commands: Commands,
) {
    if !history.has_gesture() {
        return;
    }
    let escape = keyboard.just_pressed(KeyCode::Escape);
    let focused = windows.iter().all(|window| window.focused);
    if escape || !focused {
        commands.queue(move |world: &mut World| {
            world.resource_scope(|world, mut history: Mut<EditorHistory>| {
                if escape {
                    history.cancel_gesture(world);
                } else {
                    history.finish_gesture(world);
                }
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editing::history::{open_document, HistoryPlugin};
    use crate::id_index::IdIndexPlugin;
    use bevy::ecs::system::RunSystemOnce;
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

    fn frame(
        app: &mut App,
        ctx: &Context,
        entity: Entity,
        events: Vec<Event>,
        visible: bool,
    ) -> Rect {
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
                        let mut note = *app.world().get::<Note>(entity).unwrap();
                        let response = ui.add(egui::DragValue::new(&mut note.x).speed(1));
                        rect = response.rect;
                        app.world_mut()
                            .run_system_once_with(
                                move |In(response): In<Response>, mut edits: Edits| {
                                    gesture_response(
                                        &mut edits,
                                        &response,
                                        "edit x",
                                        move |commands| {
                                            commands.entity(entity).insert(note);
                                        },
                                    );
                                },
                                response,
                            )
                            .unwrap();
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
}
