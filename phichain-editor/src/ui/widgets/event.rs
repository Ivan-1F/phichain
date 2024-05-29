use egui::{Align2, Color32, FontId, Response, Sense, Ui};
use phichain_chart::event::LineEvent;

pub fn event_ui(ui: &mut Ui, rect: egui::Rect, event: &LineEvent, color: Color32) -> Response {
    let response = ui.allocate_rect(rect, Sense::click());
    if ui.is_rect_visible(rect) {
        ui.painter()
            .rect(rect, 0.0, color, egui::Stroke::new(2.0, Color32::WHITE));
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
