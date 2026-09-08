use super::changes::{PendingChanges, Recorder};
use super::{run_commands, EditorHistory, Edits};
use bevy::prelude::*;
use uuid::Uuid;

/// Identifies one interaction, even when its widget is reused for another edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GestureId(Uuid);

pub(super) struct ActiveGesture {
    pub id: GestureId,
    pub description: String,
    pub pending: PendingChanges,
    pub seen: bool,
}

impl EditorHistory {
    pub fn has_gesture(&self) -> bool {
        self.gesture.is_some()
    }

    pub fn finish_gesture(&mut self, world: &mut World) {
        if let Some(gesture) = self.gesture.take() {
            let changes = gesture.pending.settle(world);
            self.record_changes(world, gesture.description, changes);
        }
    }

    pub fn cancel_gesture(&mut self, world: &mut World) {
        if let Some(gesture) = self.gesture.take() {
            let changes = gesture.pending.settle(world);
            changes
                .validate(world, false)
                .expect("gesture references must remain valid");
            changes.apply(world, false);
        }
    }
}

impl Edits<'_, '_> {
    pub fn begin_gesture(&mut self, description: impl Into<String>) -> GestureId {
        let id = GestureId(Uuid::new_v4());
        let description = description.into();
        self.queue(move |world| {
            world.resource_scope(|world, mut history: Mut<EditorHistory>| {
                history.finish_gesture(world);
                history.gesture = Some(ActiveGesture {
                    id,
                    description,
                    pending: PendingChanges::default(),
                    seen: true,
                });
            });
        });
        id
    }

    /// Apply a live update, retaining the first old value across all updates.
    /// Updates from an interaction that has already ended are ignored.
    pub fn gesture(&mut self, id: GestureId, edit: impl FnOnce(&mut Commands) + Send + 'static) {
        self.queue(move |world| {
            world.flush();
            if !world
                .resource::<EditorHistory>()
                .gesture
                .as_ref()
                .is_some_and(|gesture| gesture.id == id)
            {
                return;
            }
            let mut gesture = world
                .resource_mut::<EditorHistory>()
                .gesture
                .take()
                .unwrap();
            world.resource_mut::<Recorder>().resume(gesture.pending);
            run_commands(world, edit);
            gesture.pending = world.resource_mut::<Recorder>().finish();
            gesture.seen = true;
            world.resource_mut::<EditorHistory>().gesture = Some(gesture);
        });
    }

    pub fn finish_gesture(&mut self, id: GestureId) {
        self.queue(move |world| {
            world.resource_scope(|world, mut history: Mut<EditorHistory>| {
                if history
                    .gesture
                    .as_ref()
                    .is_some_and(|gesture| gesture.id == id)
                {
                    history.finish_gesture(world);
                }
            });
        });
    }

    pub fn cancel_gesture(&mut self, id: GestureId) {
        self.queue(move |world| {
            world.resource_scope(|world, mut history: Mut<EditorHistory>| {
                if history
                    .gesture
                    .as_ref()
                    .is_some_and(|gesture| gesture.id == id)
                {
                    history.cancel_gesture(world);
                }
            });
        });
    }

    /// Call while the interaction's UI is present, including frames without changes.
    pub fn keep_gesture_alive(&mut self, id: GestureId) {
        self.queue(move |world| {
            if let Some(gesture) = &mut world.resource_mut::<EditorHistory>().gesture {
                if gesture.id == id {
                    gesture.seen = true;
                }
            }
        });
    }
}

pub(super) fn finish_unattended_gesture(world: &mut World) {
    if !world.resource::<EditorHistory>().has_gesture() {
        return;
    }
    world.resource_scope(|world, mut history: Mut<EditorHistory>| {
        if let Some(gesture) = &mut history.gesture {
            if gesture.seen {
                gesture.seen = false;
            } else {
                history.finish_gesture(world);
            }
        }
    });
}
