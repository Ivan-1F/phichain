use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Resource))]
pub struct Offset(pub f32);
