use bevy::app::App;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::{Component, Plugin, Resource};
use phichain_chart::line::Line;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Resource, Default)]
struct OrderGen(AtomicU64);
impl OrderGen {
    #[inline]
    fn next(&self) -> u64 {
        self.0.fetch_add(1, Ordering::Relaxed)
    }
}

/// This is a temporary workaround to maintain line order
#[derive(Component, Default, Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
#[component(on_add = LineOrder::on_add)]
pub struct LineOrder(pub u64);

impl LineOrder {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let next = world.resource::<OrderGen>().next();
        if let Some(mut order) = world.entity_mut(ctx.entity).get_mut::<LineOrder>() {
            *order = LineOrder(next);
        }
    }
}

pub struct LinePlugin;

impl Plugin for LinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OrderGen>()
            .register_required_components::<Line, LineOrder>();
    }
}
