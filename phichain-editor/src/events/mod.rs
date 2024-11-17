use crate::events::line::LineEventPlugin;
use bevy::app::{App, Plugin};

pub mod line;

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LineEventPlugin);
    }
}
