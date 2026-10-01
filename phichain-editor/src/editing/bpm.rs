use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use phichain_chart::bpm_list::{BpmList, BpmPoint};
use phichain_chart::id::BpmPointId;

pub struct BpmEditingPlugin;

impl Plugin for BpmEditingPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(|_: On<Insert, BpmPoint>, mut commands: Commands| {
            commands.queue(rebuild);
        })
        .add_observer(|_: On<Remove, BpmPoint>, mut commands: Commands| {
            commands.queue(rebuild);
        });
    }
}

fn rebuild(world: &mut World) {
    if !world.contains_resource::<BpmList>() {
        return;
    }
    world
        .run_system_once(
            |points: Query<(&BpmPointId, &BpmPoint)>, mut list: ResMut<BpmList>| {
                let mut points: Vec<_> = points.iter().collect();
                points.sort_by_key(|(id, point)| (point.beat, **id));
                *list = BpmList::new(points.into_iter().map(|(_, point)| *point).collect());
            },
        )
        .unwrap();
}

/// Create editable BPM points from the loaded chart's timing data.
pub fn load(world: &mut World) {
    let points = world.resource::<BpmList>().0.clone();
    for point in points {
        world.spawn(point);
    }
    world.flush();
}

pub fn unload(world: &mut World) {
    let points: Vec<_> = world
        .query_filtered::<Entity, With<BpmPoint>>()
        .iter(world)
        .collect();
    for entity in points {
        world.entity_mut(entity).despawn();
    }
}
