#[cfg(feature = "bevy")]
use bevy::ecs::reflect::ReflectComponent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Resource))]
#[cfg_attr(
    feature = "bevy",
    derive(bevy::prelude::Component, bevy::prelude::Reflect)
)]
#[cfg_attr(feature = "bevy", component(immutable))]
#[cfg_attr(feature = "bevy", reflect(Component, Clone, PartialEq, Debug))]
pub struct Offset(pub f32);
