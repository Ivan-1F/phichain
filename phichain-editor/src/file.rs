use std::{marker::PhantomData, path::PathBuf};

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;
use rfd::FileDialog;

/// A file picking result event, parameterized by a marker type to distinguish different pick operations.
#[derive(Event, Debug)]
pub struct FilePickResult<M: Send + Sync + 'static> {
    pub path: Option<PathBuf>,
    _marker: PhantomData<M>,
}

impl<M: Send + Sync + 'static> FilePickResult<M> {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self {
            path,
            _marker: PhantomData,
        }
    }
}

#[derive(Component)]
struct PendingPicking<M: Send + Sync + 'static> {
    task: Task<Option<PathBuf>>,
    _marker: PhantomData<M>,
}

fn poll_system<M: Send + Sync + 'static>(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut PendingPicking<M>)>,
) {
    for (entity, mut pending) in &mut tasks {
        if let Some(path) = future::block_on(future::poll_once(&mut pending.task)) {
            commands.entity(entity).despawn();
            commands.trigger(FilePickResult::<M>::new(path));
        }
    }
}

pub trait FilePickingAppExt {
    fn register_picking_event<M: Send + Sync + 'static>(&mut self) -> &mut Self;
}

impl FilePickingAppExt for App {
    fn register_picking_event<M: Send + Sync + 'static>(&mut self) -> &mut Self {
        self.add_systems(Update, poll_system::<M>)
    }
}

pub fn pick_folder<M: Send + Sync + 'static>(world: &mut World, dialog: FileDialog) {
    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move { dialog.pick_folder() });
    world.spawn(PendingPicking::<M> {
        task,
        _marker: PhantomData,
    });
}

pub fn pick_file<M: Send + Sync + 'static>(world: &mut World, dialog: FileDialog) {
    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move { dialog.pick_file() });
    world.spawn(PendingPicking::<M> {
        task,
        _marker: PhantomData,
    });
}
