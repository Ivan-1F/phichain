use egui::Rect;

pub fn keep_aspect_ratio(container: Rect, ratio: f32) -> Rect {
    let container_ratio = container.width() / container.height();

    let (new_width, new_height) = if container_ratio > ratio {
        let height = container.height();
        let width = (height * ratio).min(container.width());
        (width, height)
    } else {
        let width = container.width();
        let height = (width / ratio).min(container.height());
        (width, height)
    };

    let x = container.min.x + (container.width() - new_width) / 2.0;
    let y = container.min.y + (container.height() - new_height) / 2.0;

    Rect::from_min_size(egui::pos2(x, y), egui::vec2(new_width, new_height))
}
