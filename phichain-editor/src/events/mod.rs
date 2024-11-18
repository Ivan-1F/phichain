use crate::events::line::LineEventPlugin;
use bevy::app::{App, Plugin};
use bevy::ecs::system::SystemState;
use bevy::log::debug;
use bevy::prelude::{Event, EventReader, IntoSystemConfigs, Update, World};
use phichain_game::GameSet;
use std::fmt::Debug;

pub mod line;

pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LineEventPlugin);
    }
}

/// A event that can be run directly on a world
pub trait EditorEvent: Event + Clone + Debug {
    /// The output of the event, only available when directly running
    type Output;

    fn run(self, world: &mut World) -> Self::Output;
}

fn handle_editor_event_system<T>(world: &mut World)
where
    T: EditorEvent,
{
    let mut state = SystemState::<EventReader<T>>::new(world);
    let mut event_reader = state.get_mut(world);
    let events = event_reader.read().cloned().collect::<Vec<_>>();
    event_reader.clear();
    for event in events {
        debug!(
            "[handle_editor_event_system<{}>] running editor event through global handler: {:?}",
            std::any::type_name::<T>(),
            event
        );
        event.run(world);
    }
}

pub trait EditorEventAppExt {
    fn add_editor_event<T>(&mut self) -> &mut Self
    where
        T: EditorEvent;
}

impl EditorEventAppExt for App {
    fn add_editor_event<T>(&mut self) -> &mut Self
    where
        T: EditorEvent,
    {
        self.add_event::<T>()
            .add_systems(Update, handle_editor_event_system::<T>.in_set(GameSet));

        self
    }
}