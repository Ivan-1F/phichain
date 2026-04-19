use egui::{Color32, Frame, Response, Sense, Stroke, Ui, UiBuilder, Vec2};

/// A clickable frame styled like [`egui::Button`], hosting arbitrary contents.
pub fn button_frame(
    ui: &mut Ui,
    selected: bool,
    add_contents: impl FnOnce(&mut Ui, Color32),
) -> Response {
    ui.scope_builder(UiBuilder::new().sense(Sense::click()), |ui| {
        ui.style_mut().interaction.selectable_labels = false;

        let response = ui.response();
        let visuals = ui.style().interact_selectable(&response, selected);

        let active =
            selected || response.hovered() || response.highlighted() || response.has_focus();
        let (fill, stroke) = if active {
            (visuals.bg_fill, visuals.bg_stroke)
        } else {
            (Color32::TRANSPARENT, Stroke::NONE)
        };

        let base = ui.spacing().button_padding;
        let inner_margin = base + Vec2::splat(visuals.expansion) - Vec2::splat(stroke.width);
        let outer_margin = -Vec2::splat(visuals.expansion);

        Frame::new()
            .fill(fill)
            .stroke(stroke)
            .inner_margin(inner_margin)
            .outer_margin(outer_margin)
            .corner_radius(visuals.corner_radius)
            .show(ui, |ui| {
                add_contents(ui, visuals.text_color());
            });
    })
    .response
}
