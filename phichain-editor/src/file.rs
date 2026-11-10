use std::{marker::PhantomData, path::PathBuf};

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;
use rfd::FileDialog;

/// A file picking result event, parameterized by a marker type to distinguish different pick operations.
#[derive(Event, Debug)]
pub struct PickedFile<M: Send + Sync + 'static> {
    pub path: Option<PathBuf>,
    _marker: PhantomData<M>,
}

impl<M: Send + Sync + 'static> PickedFile<M> {
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
            commands.trigger(PickedFile::<M>::new(path));
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

/// While present, picking leaves a [`PendingDialog`] for the program to answer instead of opening a system dialog
#[cfg(feature = "capture")]
#[derive(Resource)]
pub struct NoSystemDialogs;

/// A file dialog opened while [`NoSystemDialogs`] is present
#[cfg(feature = "capture")]
#[derive(Component)]
pub struct PendingDialog {
    respond: fn(&mut Commands, Option<PathBuf>),
}

#[cfg(feature = "capture")]
impl PendingDialog {
    /// Responds as the system dialog would once the user picked `path`, or cancelled with `None`
    pub fn answer(&self, commands: &mut Commands, path: Option<PathBuf>) {
        (self.respond)(commands, path);
    }
}

pub fn pick_folder<M: Send + Sync + 'static>(world: &mut World, dialog: FileDialog) {
    pick::<M>(world, move || dialog.pick_folder());
}

pub fn pick_file<M: Send + Sync + 'static>(world: &mut World, dialog: FileDialog) {
    pick::<M>(world, move || dialog.pick_file());
}

fn pick<M: Send + Sync + 'static>(
    world: &mut World,
    show: impl FnOnce() -> Option<PathBuf> + Send + 'static,
) {
    #[cfg(feature = "capture")]
    if world.contains_resource::<NoSystemDialogs>() {
        world.spawn(PendingDialog {
            respond: |commands, path| commands.trigger(PickedFile::<M>::new(path)),
        });
        return;
    }
    let task = AsyncComputeTaskPool::get().spawn(async move { show() });
    world.spawn(PendingPicking::<M> {
        task,
        _marker: PhantomData,
    });
}
