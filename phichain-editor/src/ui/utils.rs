use egui::epaint::PathShape;
use egui::{emath, Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use phichain_chart::easing::Easing;

/// Draw a easing curve with a [`Ui`] on the given [`Rect`]
pub fn draw_easing(ui: &mut Ui, rect: Rect, easing: Easing) -> Response {
    let response = ui.allocate_rect(rect, Sense::hover());
    let painter = ui.painter_at(rect);

    let to_screen =
        emath::RectTransform::from_to(Rect::from_min_size(Pos2::ZERO, Vec2::new(1.0, 1.0)), rect);

    let points: Vec<_> = std::iter::repeat(0.0)
        .take(40)
        .enumerate()
        .map(|(i, _)| {
            let x = i as f32 / 40.0;
            Pos2::new(x, 1.0 - easing.ease(x))
        })
        .map(|x| to_screen * x)
        .collect();

    painter.add(PathShape::line(points, Stroke::new(2.0, Color32::WHITE)));

    response
}
