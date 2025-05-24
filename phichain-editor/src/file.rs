use std::path::PathBuf;

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;
use rfd::FileDialog;

#[derive(Component)]
struct PendingPicking {
    task: Task<Option<PathBuf>>,
    kind: PickingKind,
}

#[derive(Debug, Clone, Copy)]
pub enum PickingKind {
    OpenProject,
    SelectIllustration,
    SelectMusic,
    CreateProject,
    ExportOfficial,
}

#[derive(Event, Debug)]
pub struct PickingEvent {
    pub path: Option<PathBuf>,
    pub kind: PickingKind,
}

pub struct FilePickingPlugin;

impl Plugin for FilePickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PickingEvent>()
            .add_systems(Update, poll_system);
    }
}

fn poll_system(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut PendingPicking)>,
    mut events: EventWriter<PickingEvent>,
) {
    for (entity, mut pending) in &mut tasks {
        if let Some(path) = future::block_on(future::poll_once(&mut pending.task)) {
            commands.entity(entity).despawn();
            events.write(PickingEvent {
                path,
                kind: pending.kind,
            });
        }
    }
}

// API

pub fn pick_folder(world: &mut World, kind: PickingKind, dialog: FileDialog) {
    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move { dialog.pick_folder() });
    world.spawn(PendingPicking { task, kind });
}

pub fn pick_file(world: &mut World, kind: PickingKind, dialog: FileDialog) {
    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move { dialog.pick_file() });
    world.spawn(PendingPicking { task, kind });
}
