//! Editable project settings live on the session entity. The Offset resource
//! and Project.meta mirror them for playback, serialization and export.

use bevy::prelude::*;
use phichain_chart::id::ProjectId;
use phichain_chart::offset::Offset;
use phichain_chart::project::{Project, ProjectMeta};

pub struct ProjectSettingsPlugin;

impl Plugin for ProjectSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(
            |event: On<Insert, Offset>,
             values: Query<&Offset, With<ProjectId>>,
             mut offset: ResMut<Offset>| {
                if let Ok(value) = values.get(event.entity) {
                    *offset = *value;
                }
            },
        )
        .add_observer(
            |event: On<Insert, ProjectMeta>,
             values: Query<&ProjectMeta, With<ProjectId>>,
             mut project: ResMut<Project>| {
                if let Ok(value) = values.get(event.entity) {
                    project.meta = value.clone();
                }
            },
        );
    }
}

pub fn load(world: &mut World) {
    let project = world.resource::<Project>();
    let settings = (
        ProjectId::from_uuid(project.id),
        project.meta.clone(),
        *world.resource::<Offset>(),
    );
    world.spawn(settings);
}

pub fn unload(world: &mut World) {
    let entities: Vec<_> = world
        .query_filtered::<Entity, With<ProjectId>>()
        .iter(world)
        .collect();
    for entity in entities {
        world.entity_mut(entity).despawn();
    }
}
