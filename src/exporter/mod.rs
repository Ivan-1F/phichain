use bevy::prelude::*;

pub mod phichain;

pub trait Exporter {
    fn export(world: &mut World) -> anyhow::Result<String>;
}
