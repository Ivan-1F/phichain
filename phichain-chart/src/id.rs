use std::ops::{Deref, DerefMut};

use uuid::Uuid;

/// A unique identifier for game objects in the form of uuid. This should be applied to every [`Note`] and [`Line`]
///
/// Avoid using this to find entities in systems running on [`Update`] as it is computationally expensive
///
/// This is intended for one-time operations such as undo, redo and export
#[derive(Debug, Clone)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct Id(Uuid);

impl Id {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Id {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Id {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
