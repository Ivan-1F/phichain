pub mod modifier;
pub mod next;
pub mod record;

use bevy::prelude::*;

pub struct HotkeyPlugin;

impl Plugin for HotkeyPlugin {
    fn build(&self, _: &mut App) {}
}
