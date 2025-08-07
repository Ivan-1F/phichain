use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
#[cfg_attr(
    feature = "bevy",
    require(
        bevy::prelude::Sprite,
        LinePosition,
        LineRotation,
        LineOpacity,
        LineSpeed,
        LineTimestamp
    )
)]
pub struct Line {
    pub name: String,
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

/// This is a temporary workaround to maintain line order
///
/// TODO: remove this when game-object-id is merged
#[cfg(feature = "bevy")]
#[derive(bevy::prelude::Component, Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct LineTimestamp(pub u64);

#[cfg(feature = "bevy")]
impl Default for LineTimestamp {
    fn default() -> Self {
        Self(
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        )
    }
}
