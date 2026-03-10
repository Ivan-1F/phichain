use std::{marker::PhantomData, path::PathBuf};

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use futures_lite::future;
use rfd::FileDialog;

pub trait FilePickingResult: Event {
    fn new(path: Option<PathBuf>) -> Self;
}

macro_rules! picking_event {
    ($name:ident) => {
        #[derive(::bevy::prelude::Event, Debug)]
        pub struct $name(pub Option<::std::path::PathBuf>);

        impl $crate::file::FilePickingResult for $name {
            fn new(path: Option<::std::path::PathBuf>) -> Self {
                Self(path)
            }
        }
    };
}

pub(crate) use picking_event;

#[derive(Component)]
struct PendingPicking<E: FilePickingResult> {
    task: Task<Option<PathBuf>>,
    _marker: PhantomData<E>,
}

fn poll_system<E: FilePickingResult>(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut PendingPicking<E>)>,
) {
    for (entity, mut pending) in &mut tasks {
        if let Some(path) = future::block_on(future::poll_once(&mut pending.task)) {
            commands.entity(entity).despawn();
            commands.trigger(E::new(path));
        }
    }
}

pub trait FilePickingAppExt {
    fn register_picking_event<E: FilePickingResult>(&mut self) -> &mut Self;
}

impl FilePickingAppExt for App {
    fn register_picking_event<E: FilePickingResult>(&mut self) -> &mut Self {
        self.add_systems(Update, poll_system::<E>)
    }
}

pub fn pick_folder<E: FilePickingResult>(world: &mut World, dialog: FileDialog) {
    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move { dialog.pick_folder() });
    world.spawn(PendingPicking::<E> {
        task,
        _marker: PhantomData,
    });
}

pub fn pick_file<E: FilePickingResult>(world: &mut World, dialog: FileDialog) {
    let thread_pool = AsyncComputeTaskPool::get();
    let task = thread_pool.spawn(async move { dialog.pick_file() });
    world.spawn(PendingPicking::<E> {
        task,
        _marker: PhantomData,
    });
}
