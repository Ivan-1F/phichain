use crate::selection::SelectEvent;
use crate::timeline::{Timeline, TimelineContext};
use crate::utils::convert::BevyEguiConvert;
use bevy::app::App;
use bevy::ecs::system::SystemState;
use bevy::prelude::{EventWriter, Plugin, Query, ResMut, Resource, Window, World};
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
    let (ctx, mut selection, window_query) = state.get_mut(world);
    let viewport = ctx.viewport.0;
    let window = window_query.single();
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let response = ui.allocate_rect(viewport.into_egui(), Sense::drag());

    let calculate_x = || cursor_position.x - viewport.min.x;
    let calculate_time = || ctx.y_to_time(cursor_position.y);

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
        let start_y = ctx.time_to_y(start.y);
        let now_x = viewport.min.x + now.x;
        let now_y = ctx.time_to_y(now.y);
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
        if let Some((from, to)) = selection.0.take() {
            let selection_rect = egui::Rect::from_two_pos(from.to_pos2(), to.to_pos2())
                .translate(egui::Vec2::new(ctx.viewport.0.min.x, 0.0));
            // ignore too small selections. e.g. click on a note
            if selection_rect.area() >= 0.001 {
                let timelines = ctx.settings.container.clone();
                let mut all = vec![];
                for item in timelines.allocate(ctx.viewport.0.into_egui()) {
                    let rect =
                        selection_rect.with_min_x(selection_rect.min.x.max(item.viewport.left()));
                    let rect = rect.with_max_x(rect.max.x.min(item.viewport.right()));
                    let selected = item.timeline.on_drag_selection(
                        world,
                        item.viewport,
                        rect.translate(egui::Vec2::new(-item.viewport.left(), 0.0)),
                    );
                    all.extend(selected);
                }
                let mut state: SystemState<EventWriter<SelectEvent>> = SystemState::new(world);
                let mut select_events = state.get_mut(world);
                select_events.send(SelectEvent(all));
            }
        }
    }
}
