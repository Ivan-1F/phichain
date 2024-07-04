use crate::timeline::TimelineContext;
use bevy::app::App;
use bevy::ecs::system::SystemState;
use bevy::prelude::{Plugin, Query, ResMut, Resource, Window, World};
use egui::{Color32, Sense, Stroke, Ui};

/// Represents the drag-selection on the timeline
#[derive(Resource, Debug, Default)]
pub struct TimelineDragSelection(pub Option<(egui::Vec2, egui::Vec2)>);

pub struct TimelineDragSelectionPlugin;

impl Plugin for TimelineDragSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimelineDragSelection::default());
    }
}

pub fn timeline_drag_selection(ui: &mut Ui, world: &mut World) {
    let mut state: SystemState<(
        TimelineContext,
        ResMut<TimelineDragSelection>,
        Query<&Window>,
    )> = SystemState::new(world);
    let (timeline, mut selection, window_query) = state.get_mut(world);
    let viewport = timeline.viewport.0;
    let window = window_query.single();
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let response = ui.allocate_rect(
        egui::Rect::from_min_max(
            egui::Pos2::new(viewport.min.x, viewport.min.y),
            egui::Pos2::new(viewport.max.x, viewport.max.y),
        ),
        Sense::drag(),
    );

    let calculate_x = || cursor_position.x - viewport.min.x;
    let calculate_time = || timeline.y_to_time(cursor_position.y);

    if response.drag_started() {
        let (x, time) = (calculate_x(), calculate_time());
        selection.0 = Some((egui::Vec2::new(x, time), egui::Vec2::new(x, time)));
    }

    if response.dragged() {
        let (x, time) = (calculate_x(), calculate_time());
        selection.0 = Some((selection.0.unwrap().0, egui::Vec2::new(x, time)));
    }

    if let Some((start, now)) = selection.0 {
        let start_x = viewport.min.x + start.x;
        let start_y = timeline.time_to_y(start.y);
        let now_x = viewport.min.x + now.x;
        let now_y = timeline.time_to_y(now.y);
        ui.painter().rect(
            egui::Rect::from_two_pos(
                egui::Pos2::new(start_x, start_y),
                egui::Pos2::new(now_x, now_y),
            ),
            0.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 20),
            Stroke::NONE,
        );
    }

    if response.drag_stopped() {
        if let Some((from, to)) = selection.0 {
            let rect = egui::Rect::from_two_pos(from.to_pos2(), to.to_pos2());
            // ignore too small selections. e.g. click on a note
            if rect.area() >= 0.001 {
                // TODO: broadcast this to all timelines
                // let x_range = rect.x_range();
                // let time_range = rect.y_range();
                //
                // let notes = note_query
                //     .iter()
                //     .filter(|x| x.1.get() == selected_line.0)
                //     .filter(|x| {
                //         let note = x.0;
                //         x_range.contains(note.x) && time_range.contains(bpm_list.time_at(note.beat))
                //     })
                //     .map(|x| x.2)
                //     .collect::<Vec<_>>();
                //
                // select_events.send(SelectEvent(notes));
            }
        }
        selection.0 = None;
    }
}
