//! Explicit edits with automatically recorded document changes.
//!
//! Commands execute inside the recording scope. History stores their results;
//! undo and redo never execute the original editing closure again.
//!
//! Registered document components are immutable: replace them with `insert`
//! inside `Edits::once` or `Edits::gesture`, and use `despawn` to delete an object.
//! Its identity must stay fixed. Loading, teardown, `Pending`, and `Derived` are excluded.

mod changes;
mod gesture;
pub use gesture::GestureId;
use gesture::{finish_unattended_gesture, ActiveGesture};
#[cfg(test)]
mod tests;

use self::changes::{ChangeSet, Recorder, Registry};
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use phichain_chart::bpm_list::BpmPoint;
use phichain_chart::curve_note_track::CurveNoteTrackOptions;
use phichain_chart::event::LineEvent;
use phichain_chart::id::{BpmPointId, CurveNoteTrackId, EventId, LineId, NoteId};
use phichain_chart::line::Line;
use phichain_chart::note::Note;
use phichain_game::curve_note_track::{CurveNoteTrackFrom, CurveNoteTrackTo};
use phichain_game::event::EventOf;
use phichain_game::line::LineOrder;
use std::fmt;
use undo::{Edit, Record};
use uuid::Uuid;

/// Limit complete user edits, rather than individual component changes.
pub const MAX_HISTORY_ENTRIES: usize = 1_000;

pub struct HistoryPlugin;

impl Plugin for HistoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AppTypeRegistry>()
            .init_resource::<Registry>()
            .init_resource::<Recorder>()
            .init_resource::<EditorHistory>()
            .add_systems(Last, finish_unattended_gesture);
        changes::register_component::<NoteId>(app, true);
        changes::register_component::<Note>(app, false);
        changes::register_component::<ChildOf>(app, false);
        changes::register_component::<EventId>(app, true);
        changes::register_component::<LineEvent>(app, false);
        changes::register_component::<EventOf>(app, false);
        changes::register_component::<LineId>(app, true);
        changes::register_component::<Line>(app, false);
        changes::register_component::<LineOrder>(app, false);
        changes::register_component::<CurveNoteTrackId>(app, true);
        changes::register_component::<CurveNoteTrackOptions>(app, false);
        changes::register_component::<CurveNoteTrackFrom>(app, false);
        changes::register_component::<CurveNoteTrackTo>(app, false);
        changes::register_component::<BpmPointId>(app, true);
        changes::register_component::<BpmPoint>(app, false);
    }
}

/// The only active history for the open project.
#[derive(Resource)]
pub struct EditorHistory {
    record: Record<RecordedEdit>,
    current: Uuid,
    epoch: Uuid,
    gesture: Option<ActiveGesture>,
}

impl Default for EditorHistory {
    fn default() -> Self {
        Self {
            record: Record::builder().limit(MAX_HISTORY_ENTRIES).build(),
            current: Uuid::new_v4(),
            epoch: Uuid::new_v4(),
            gesture: None,
        }
    }
}

impl EditorHistory {
    pub fn is_saved(&self) -> bool {
        self.record.is_saved()
            && self
                .gesture
                .as_ref()
                .is_none_or(|gesture| gesture.pending.is_empty())
    }

    pub fn set_saved(&mut self) {
        assert!(
            !self.has_gesture(),
            "finish the active gesture before saving"
        );
        self.record.set_saved();
    }

    /// A state identity, not a stack index (indices are reused after undo or eviction).
    pub fn head(&self) -> Uuid {
        self.current
    }

    pub fn undo(&mut self, world: &mut World) -> anyhow::Result<()> {
        if self.has_gesture() {
            self.cancel_gesture(world);
            return Ok(());
        }
        if !self.record.can_undo() {
            return Ok(());
        }
        let entry = self.record.get_entry(self.record.head() - 1).unwrap();
        entry.as_ref().changes.validate(world, false)?;
        self.current = self.record.undo(world).unwrap();
        Ok(())
    }

    pub fn redo(&mut self, world: &mut World) -> anyhow::Result<()> {
        if self.has_gesture() {
            self.cancel_gesture(world);
            return Ok(());
        }
        if !self.record.can_redo() {
            return Ok(());
        }
        let entry = self.record.get_entry(self.record.head()).unwrap();
        entry.as_ref().changes.validate(world, true)?;
        self.current = self.record.redo(world).unwrap();
        Ok(())
    }

    fn record_changes(&mut self, world: &mut World, description: String, changes: ChangeSet) {
        if changes.is_empty() {
            return;
        }
        let revision_after = Uuid::new_v4();
        let entry = RecordedEdit {
            description,
            changes,
            revision_before: self.current,
            revision_after,
        };
        debug!("Committed edit: {}", entry);
        self.record.edit(world, entry);
        self.current = revision_after;
    }
}

struct RecordedEdit {
    description: String,
    changes: ChangeSet,
    revision_before: Uuid,
    revision_after: Uuid,
}

impl fmt::Display for RecordedEdit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.description)
    }
}

impl Edit for RecordedEdit {
    type Target = World;
    type Output = Uuid;

    fn edit(&mut self, _: &mut World) -> Uuid {
        // The initial edit has already happened inside the recording scope.
        self.revision_after
    }

    fn undo(&mut self, world: &mut World) -> Uuid {
        self.changes.apply(world, false);
        self.revision_before
    }

    fn redo(&mut self, world: &mut World) -> Uuid {
        self.changes.apply(world, true);
        self.revision_after
    }
}

#[derive(SystemParam)]
pub struct Edits<'w, 's> {
    commands: Commands<'w, 's>,
    history: Res<'w, EditorHistory>,
}

impl Edits<'_, '_> {
    /// Queue one user operation. Its commands and synchronous observer commands
    /// form one history entry; an operation with no document changes is discarded.
    pub fn once(
        &mut self,
        description: impl Into<String>,
        edit: impl FnOnce(&mut Commands) + Send + 'static,
    ) {
        let description = description.into();
        self.queue(move |world| {
            world.resource_scope(|world, mut history: Mut<EditorHistory>| {
                history.finish_gesture(world)
            });
            apply_edit(world, description, edit);
        });
    }

    fn queue(&mut self, edit: impl FnOnce(&mut World) + Send + 'static) {
        let epoch = self.history.epoch;
        self.commands.queue(move |world: &mut World| {
            // A queued UI action must not leak into another project.
            if world.resource::<EditorHistory>().epoch == epoch
                && world.resource::<Recorder>().enabled
            {
                edit(world);
            }
        });
    }
}

fn apply_edit(world: &mut World, description: String, edit: impl FnOnce(&mut Commands)) {
    // Flush unrelated work before opening the recording scope.
    world.flush();
    world.resource_mut::<Recorder>().begin();
    run_commands(world, edit);
    let pending = world.resource_mut::<Recorder>().finish();
    let changes = pending.settle(world);
    world.resource_scope(|world, mut history: Mut<EditorHistory>| {
        history.record_changes(world, description, changes);
    });
}

fn run_commands(world: &mut World, edit: impl FnOnce(&mut Commands)) {
    let mut queue = CommandQueue::default();
    edit(&mut Commands::new(&mut queue, world));
    queue.apply(world);
    // Include commands issued by synchronous lifecycle observers.
    world.flush();
}

/// Called only after the initial document has finished loading.
pub fn open_document(world: &mut World) {
    world.flush();
    *world.resource_mut::<EditorHistory>() = EditorHistory::default();
    *world.resource_mut::<Recorder>() = Recorder::default();
    world.resource_mut::<Recorder>().enabled = true;
}

/// Stop recording before teardown, invalidate queued edits, and release history data.
pub fn close_document(world: &mut World) {
    *world.resource_mut::<Recorder>() = Recorder::default();
    *world.resource_mut::<EditorHistory>() = EditorHistory::default();
}
