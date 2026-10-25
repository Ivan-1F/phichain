use bevy::prelude::*;
use phichain_chart::line::Line;

#[derive(Resource, Default)]
struct OrderGen(u64);

/// The order of a line among its siblings.
#[derive(Component, Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Reflect)]
#[component(immutable)]
#[reflect(Component, Clone, PartialEq, Debug)]
pub struct LineOrder(pub u64);

pub struct LinePlugin;

impl Plugin for LinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OrderGen>()
            .add_observer(assign_line_order);
    }
}

fn assign_line_order(event: On<Add, Line>, mut order: ResMut<OrderGen>, mut commands: Commands) {
    let next = order.0;
    order.0 += 1;
    // A restored or explicitly ordered line keeps its own position.
    commands
        .entity(event.entity)
        .try_insert_if_new(LineOrder(next));
}
