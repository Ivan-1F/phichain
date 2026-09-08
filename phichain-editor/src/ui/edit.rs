pub mod component;
pub use component::{EditResponse, EditWidget};

use crate::editing::history::{EditorHistory, Edits, GestureId};
use bevy::prelude::*;
use egui::{Event, Key, Response};

/// Connect a value-changing widget to history. Only interaction identity lives
/// in egui memory; old document values are retained by the ECS recorder.
fn gesture_response(
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
mod tests;
