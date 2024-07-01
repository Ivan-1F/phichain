use crate::tab::timeline::Timeline;
use egui::{Align2, Color32, FontId, Rangef, Rect, Response, Sense, Stroke, Ui};
use phichain_chart::event::LineEvent;

pub fn event_ui<F>(
    ui: &mut Ui,
    rect: Rect,
    event: &mut LineEvent,
    color: Color32,
    timeline: &Timeline,
    mut on_change: F,
) -> Response
where
    F: FnMut(LineEvent, LineEvent),
{
    let response = ui.allocate_rect(rect, Sense::click());
    if ui.is_rect_visible(rect) {
        ui.painter()
            .rect(rect, 0.0, color, Stroke::new(2.0, Color32::WHITE));

        let mut make_drag_zone = |start: bool| {
            let drag_zone = Rect::from_x_y_ranges(
                rect.x_range(),
                if start {
                    Rangef::from(rect.max.y - 5.0..=rect.max.y)
                } else {
                    Rangef::from(rect.min.y..=rect.min.y + 5.0)
                },
            );
            let response = ui
                .allocate_rect(drag_zone, Sense::drag())
                .on_hover_and_drag_cursor(egui::CursorIcon::ResizeVertical);

            if response.drag_started() {
                ui.data_mut(|data| data.insert_temp(egui::Id::new("event-drag"), *event));
            }

            if response.dragged() {
                let drag_delta = response.drag_delta();

                if start {
                    let new_y = timeline.beat_to_y(event.start_beat) + drag_delta.y;
                    event.start_beat = timeline.y_to_beat(new_y); // will be attached when stop dragging
                } else {
                    let new_y = timeline.beat_to_y(event.end_beat) + drag_delta.y;
                    event.end_beat = timeline.y_to_beat(new_y); // will be attached when stop dragging
                }
            }

            if response.drag_stopped() {
                let from = ui.data(|data| {
                    data.get_temp::<LineEvent>(egui::Id::new("event-drag"))
                        .unwrap()
                });
                ui.data_mut(|data| data.remove::<LineEvent>(egui::Id::new("event-drag")));
                if start {
                    event.start_beat = timeline.timeline_settings.attach(event.start_beat.value());
                } else {
                    event.end_beat = timeline.timeline_settings.attach(event.end_beat.value());
                }
                if from != *event {
                    on_change(from, *event);
                }
            }
        };

        make_drag_zone(true);
        make_drag_zone(false);

        ui.painter().text(
            rect.center_top(),
            Align2::CENTER_TOP,
            event.end,
            FontId::default(),
            Color32::DARK_GREEN,
        );
        ui.painter().text(
            rect.center_bottom(),
            Align2::CENTER_BOTTOM,
            event.start,
            FontId::default(),
            Color32::DARK_GREEN,
        );
    }

    response
}
