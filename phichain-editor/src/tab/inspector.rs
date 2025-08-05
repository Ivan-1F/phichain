mod curve_note_track;
mod line;
mod multiple_events;
mod multiple_notes;
mod single_event;
mod single_note;

use crate::tab::inspector::curve_note_track::curve_note_track_inspector;
use crate::tab::inspector::line::line_inspector;
use crate::tab::inspector::multiple_events::multiple_events_inspector;
use crate::tab::inspector::multiple_notes::multiple_notes_inspector;
use crate::tab::inspector::single_event::single_event_inspector;
use crate::tab::inspector::single_note::single_note_inspector;
use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use egui::{Ui, UiBuilder};

#[derive(Debug, Clone, Copy, Component)]
pub struct Inspector {
    system: SystemId<In<Ui>, Result>,
}

impl Inspector {
    pub fn new(system: SystemId<In<Ui>, Result>) -> Self {
        Self { system }
    }
}

pub trait InspectorRegistrationExt {
    fn add_inspector<S, M>(&mut self, inspector: S) -> &mut Self
    where
        S: IntoSystem<In<Ui>, Result, M> + 'static;
}

impl InspectorRegistrationExt for App {
    fn add_inspector<S, M>(&mut self, inspector: S) -> &mut Self
    where
        S: IntoSystem<In<Ui>, Result, M> + 'static,
    {
        let system_id = self.world_mut().register_system(inspector);
        self.world_mut().spawn(Inspector::new(system_id));
        self
    }
}

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_inspector(single_note_inspector)
            .add_inspector(multiple_notes_inspector)
            .add_inspector(curve_note_track_inspector)
            .add_inspector(single_event_inspector)
            .add_inspector(multiple_events_inspector)
            .add_inspector(line_inspector);
    }
}

pub fn inspector_ui_system(In(mut ui): In<Ui>, world: &mut World) {
    let inspectors = world
        .query::<&mut Inspector>()
        .iter(world)
        .copied()
        .collect::<Vec<_>>();

    for inspector in inspectors {
        let _ = world.run_system_with(
            inspector.system,
            ui.new_child(
                UiBuilder::new()
                    .max_rect(ui.max_rect())
                    .layout(*ui.layout()),
            ),
        );
    }
}
