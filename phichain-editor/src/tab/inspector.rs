mod curve_note_track;
mod line;
mod multiple_events;
mod multiple_notes;
mod single_event;
mod single_note;

use crate::selection::Selected;
use crate::tab::inspector::curve_note_track::curve_note_track_inspector;
use crate::tab::inspector::line::line_inspector;
use crate::tab::inspector::multiple_events::multiple_events_inspector;
use crate::tab::inspector::multiple_notes::multiple_notes_inspector;
use crate::tab::inspector::single_event::single_event_inspector;
use crate::tab::inspector::single_note::single_note_inspector;
use bevy::ecs::system::{RegisteredSystemError, SystemId, SystemParamValidationError};
use bevy::prelude::*;
use egui::{Ui, UiBuilder};
use phichain_chart::event::LineEvent;
use phichain_chart::note::Note;
use phichain_game::curve_note_track::CurveNoteTrack;

#[derive(Debug, Clone, Copy, Component)]
pub struct Inspector {
    system: SystemId<In<Ui>, Result>,
    condition: SystemId<(), bool>,
}

impl Inspector {
    pub fn new(system: SystemId<In<Ui>, Result>, condition: SystemId<(), bool>) -> Self {
        Self { system, condition }
    }
}

pub trait InspectorRegistrationExt {
    fn add_inspector<S, M, C, Marker>(&mut self, inspector: S, condition: C) -> &mut Self
    where
        S: IntoSystem<In<Ui>, Result, M> + 'static,
        C: Condition<Marker> + 'static;
}

impl InspectorRegistrationExt for App {
    fn add_inspector<S, M, C, Marker>(&mut self, inspector: S, condition: C) -> &mut Self
    where
        S: IntoSystem<In<Ui>, Result, M> + 'static,
        C: Condition<Marker> + 'static,
    {
        let system_id = self.world_mut().register_system(inspector);
        let condition_id = self.world_mut().register_system(condition);
        self.world_mut()
            .spawn(Inspector::new(system_id, condition_id));
        self
    }
}

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_inspector(single_note_inspector, single_selected::<Note>)
            .add_inspector(multiple_notes_inspector, multiple_selected::<Note>)
            .add_inspector(
                curve_note_track_inspector,
                single_selected::<CurveNoteTrack>,
            )
            .add_inspector(single_event_inspector, single_selected::<LineEvent>)
            .add_inspector(multiple_events_inspector, multiple_selected::<LineEvent>)
            .add_inspector(line_inspector, || true);
    }
}

pub fn inspector_ui_system(In(mut ui): In<Ui>, world: &mut World) {
    let inspectors = world
        .query::<&mut Inspector>()
        .iter(world)
        .copied()
        .collect::<Vec<_>>();

    for inspector in inspectors {
        let condition_met = match world.run_system(inspector.condition) {
            Ok(result) => result,
            Err(RegisteredSystemError::InvalidParams {
                err: SystemParamValidationError { skipped: true, .. },
                ..
            }) => false,
            Err(_) => {
                // TODO: add inspector name to log here
                warn!("Failed to run condition for inspector");

                false
            }
        };

        if condition_met {
            let _ = world.run_system_with(
                inspector.system,
                ui.new_child(
                    UiBuilder::new()
                        .max_rect(ui.max_rect())
                        .layout(*ui.layout()),
                ),
            );

            break;
        }
    }
}

/// A [`Condition`]-satisfying system that returns true if there's exactly one entity with given component T is selected
pub fn single_selected<T>(
    selected_query: Option<Single<Entity, With<Selected>>>,
    query: Query<&T>,
) -> bool
where
    T: Component,
{
    if let Some(entity) = selected_query {
        query.contains(*entity)
    } else {
        false
    }
}

/// A [`Condition`]-satisfying system that returns true if there's more than one entity selected, and all of them have the give component T
pub fn multiple_selected<T>(selected_query: Query<Entity, With<Selected>>, query: Query<&T>) -> bool
where
    T: Component,
{
    !selected_query.is_empty() && selected_query.iter().all(|e| query.contains(e))
}
