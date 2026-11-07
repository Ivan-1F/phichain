use crate::id::{HasId, LineId};
#[cfg(feature = "bevy")]
use bevy::ecs::reflect::ReflectComponent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
#[cfg_attr(feature = "bevy", component(immutable))]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Reflect))]
#[cfg_attr(feature = "bevy", reflect(Component, Clone, PartialEq, Debug))]
#[cfg_attr(
    feature = "bevy",
    require(
        bevy::prelude::Sprite,
        bevy::prelude::Pickable,
        LinePosition,
        LineRotation,
        LineOpacity,
        LineSpeed,
        LineId,
    )
)]
pub struct Line {
    pub name: String,
}

impl HasId for Line {
    type Id = LineId;
}

impl Default for Line {
    fn default() -> Self {
        Self {
            name: "Unnamed Line".to_owned(),
        }
    }
}

// TODO: types below should be moved to phichain-game

#[cfg(feature = "bevy")]
#[derive(bevy::prelude::Component, Debug, Default)]
pub struct LinePosition(pub bevy::prelude::Vec2);

#[cfg(feature = "bevy")]
#[derive(bevy::prelude::Component, Debug, Default)]
pub struct LineRotation(pub f32);

#[cfg(feature = "bevy")]
#[derive(bevy::prelude::Component, Debug, Default)]
pub struct LineOpacity(pub f32);

/// This will not affect line entity, it is only used to show realtime speed of lines in [phichain::tab::line_list]
#[cfg(feature = "bevy")]
#[derive(bevy::prelude::Component, Debug, Default)]
pub struct LineSpeed(pub f32);
