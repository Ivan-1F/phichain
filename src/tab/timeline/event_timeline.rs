use crate::chart::event::LineEvent;
use crate::editing::pending::Pending;
use crate::selection::{SelectEvent, Selected, SelectedLine};
use crate::tab::timeline::{Timeline, TimelineViewport};
use crate::timing::BpmList;
use crate::ui::widgets::event::event_ui;
use bevy::app::{App, Plugin};
use bevy::hierarchy::Parent;
use bevy::prelude::{Entity, EventWriter, In, Query, Res, ResMut, Resource, Window};
use egui::{Color32, Sense, Stroke, Ui};
use std::iter;

pub struct EventTimelinePlugin;

impl Plugin for EventTimelinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EventTimelineDragSelection>();
    }
}

/// Represents the drag-selection on the event timeline
#[derive(Resource, Debug, Default)]
pub struct EventTimelineDragSelection(pub Option<(egui::Vec2, egui::Vec2)>);

pub fn event_timeline_drag_select_system(
    In(ui): In<&mut Ui>,
    viewport: Res<TimelineViewport>,
    bpm_list: Res<BpmList>,
    event_query: Query<(
        &LineEvent,
        &Parent,
        Entity,
        Option<&Selected>,
        Option<&Pending>,
    )>,
    mut select_events: EventWriter<SelectEvent>,
    timeline: Timeline,

    mut selection: ResMut<EventTimelineDragSelection>,
    window_query: Query<&Window>,

    selected_line: Res<SelectedLine>,
) {
    let event_timeline_viewport = viewport.event_timeline_viewport();
    let window = window_query.single();
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let calc_event_attrs = || {
        let time = timeline.y_to_time(cursor_position.y);
        let x =
            (cursor_position.x - event_timeline_viewport.min.x) / event_timeline_viewport.width();
        (x, time)
    };

    let response = ui.allocate_rect(
        egui::Rect::from_min_max(
            egui::Pos2::new(event_timeline_viewport.min.x, event_timeline_viewport.min.y),
            egui::Pos2::new(event_timeline_viewport.max.x, event_timeline_viewport.max.y),
        ),
        Sense::drag(),
    );

    if response.drag_started() {
        let (x, time) = calc_event_attrs();
        selection.0 = Some((egui::Vec2::new(x, time), egui::Vec2::new(x, time)));
    }

    if response.dragged() {
        let (x, time) = calc_event_attrs();
        selection.0 = Some((selection.0.unwrap().0, egui::Vec2::new(x, time)));
    }

    if let Some((start, now)) = selection.0 {
        let start_x = event_timeline_viewport.min.x + start.x * event_timeline_viewport.width();
        let start_y = timeline.time_to_y(start.y);
        let now_x = event_timeline_viewport.min.x + now.x * event_timeline_viewport.width();
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
            // ignore too small selections. e.g. click on a event
            if rect.area() >= 0.001 {
                let x_range = rect.x_range();
                let time_range = rect.y_range();

                let events = event_query
                    .iter()
                    .filter(|x| x.1.get() == selected_line.0)
                    .filter(|x| {
                        let event = x.0;
                        let track: u8 = event.kind.into();
                        x_range.contains(track as f32 * (1.0 / 5.0))
                            && time_range.contains(bpm_list.time_at(event.start_beat))
                    })
                    .map(|x| x.2)
                    .collect::<Vec<_>>();

                for entity in events {
                    select_events.send(SelectEvent(entity));
                }
            }
        }
        selection.0 = None;
    }
}

pub fn event_timeline_system(
    In(ui): In<&mut Ui>,
    selected_line_query: Res<SelectedLine>,
    timeline_viewport: Res<TimelineViewport>,
    bpm_list: Res<BpmList>,
    event_query: Query<(
        &LineEvent,
        &Parent,
        Entity,
        Option<&Selected>,
        Option<&Pending>,
    )>,
    mut select_events: EventWriter<SelectEvent>,
    timeline: Timeline,
) {
    let selected_line = selected_line_query.0;
    let viewport = timeline_viewport;

    let event_timeline_viewport = viewport.event_timeline_viewport();

    // [0.2, 0.4, 0.6, 0.8]
    let lane_percents = iter::repeat(0.0)
        .take(5 - 1)
        .enumerate()
        .map(|(i, _)| (i + 1) as f32 * 1.0 / 5.0)
        .collect::<Vec<_>>();
    for percent in lane_percents {
        ui.painter().rect_filled(
            egui::Rect::from_center_size(
                egui::Pos2::new(
                    event_timeline_viewport.min.x + event_timeline_viewport.width() * percent,
                    viewport.0.center().y,
                ),
                egui::Vec2::new(2.0, viewport.0.height()),
            ),
            0.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 40),
        );
    }

    for (event, parent, entity, selected, pending) in event_query.iter() {
        if parent.get() != selected_line {
            continue;
        }

        let track: u8 = event.kind.into();

        let x = event_timeline_viewport.width() / 5.0 * track as f32
            - event_timeline_viewport.width() / 5.0 / 2.0
            + event_timeline_viewport.min.x;
        let y = timeline.time_to_y(bpm_list.time_at(event.start_beat));

        let size = egui::Vec2::new(
            event_timeline_viewport.width() / 8000.0 * 989.0,
            y - timeline.time_to_y(bpm_list.time_at(event.end_beat)),
        );

        let center = egui::Pos2::new(x, y - size.y / 2.0);

        let mut color = if selected.is_some() {
            Color32::LIGHT_GREEN
        } else {
            Color32::WHITE
        };

        if pending.is_some() {
            color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 20);
        }

        if event_ui(ui, egui::Rect::from_center_size(center, size), event, color).clicked() {
            select_events.send(SelectEvent(entity));
        }
    }
}
