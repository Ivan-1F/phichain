use egui::epaint::PathShape;
use egui::{emath, Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};
use phichain_chart::easing::Easing;
use strum::IntoEnumIterator;

pub struct EasingGraph<'a> {
    value: &'a mut Easing,
    inverse: bool,
    mirror: bool,
    color: Color32,
}

impl<'a> EasingGraph<'a> {
    pub fn new(value: &'a mut Easing) -> Self {
        Self {
            value,
            inverse: false,
            mirror: false,
            color: Color32::WHITE,
        }
    }

    pub fn inverse(mut self, reverse: bool) -> Self {
        self.inverse = reverse;
        self
    }

    pub fn mirror(mut self, mirror: bool) -> Self {
        self.mirror = mirror;
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }
}

impl Widget for EasingGraph<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut drag_stopped = false;

        let (mut response, painter) = ui.allocate_painter(
            Vec2::new(ui.available_width(), ui.available_height()),
            Sense::hover(),
        );

        draw_easing(
            ui,
            response.rect,
            *self.value,
            self.inverse,
            self.mirror,
            self.color,
        );

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, Vec2::new(1.0, 1.0)),
            response.rect,
        );

        if let Easing::Custom(x1, y1, x2, y2) = self.value {
            let mut p1 = Pos2::new(*x1, 1.0 - *y1);
            let mut p2 = Pos2::new(*x2, 1.0 - *y2);
            let size = Vec2::splat(2.0 * 4.0);

            let point_in_screen = to_screen.transform_pos(p1);
            let point_rect = Rect::from_center_size(point_in_screen, size);
            let point_id = response.id.with(1);
            let point_response = ui.interact(point_rect, point_id, Sense::drag());
            drag_stopped |= point_response.drag_stopped();

            p1 += point_response.drag_delta() / response.rect.size();
            p1 = to_screen.from().clamp(p1);

            let point_in_screen = to_screen.transform_pos(p2);
            let point_rect = Rect::from_center_size(point_in_screen, size);
            let point_id = response.id.with(2);
            let point_response = ui.interact(point_rect, point_id, Sense::drag());
            drag_stopped |= point_response.drag_stopped();

            p2 += point_response.drag_delta() / response.rect.size();
            p2 = to_screen.from().clamp(p2);

            let mut x1_ = *x1;
            let mut y1_ = *y1;
            let mut x2_ = *x2;
            let mut y2_ = *y2;

            ui.add_space(4.0); // add some space to make sure 0, 0 and drag values are not too close

            ui.horizontal(|ui| {
                // make changes affect response.drag_stopped
                macro_rules! drag_value {
                    ($($x:ident)*) => {
                        $(
                            let dv_response = ui.add(egui::DragValue::new(&mut $x).speed(0.01));
                            drag_stopped |= dv_response.drag_stopped() || dv_response.lost_focus();
                        )*
                    };
                }
                drag_value!(x1_ y1_ x2_ y2_);
            });

            if x1_ != *x1 || y1_ != *y1 || x2_ != *x2 || y2_ != *y2 {
                p1.x = x1_;
                p1.y = 1.0 - y1_;
                p2.x = x2_;
                p2.y = 1.0 - y2_;
            }

            if p1.x != *x1 || p1.y != *y1 || p2.x != *x2 || p2.y != *y2 {
                *self.value = Easing::Custom(p1.x, 1.0 - p1.y, p2.x, 1.0 - p2.y);
            }

            painter.circle(to_screen * p1, 4.0, Color32::WHITE, Stroke::NONE);
            painter.circle(to_screen * p2, 4.0, Color32::WHITE, Stroke::NONE);

            painter.line_segment(
                [to_screen * Pos2::new(0.0, 1.0), to_screen * p1],
                Stroke::new(2.0, Color32::GRAY),
            );
            painter.line_segment(
                [to_screen * Pos2::new(1.0, 0.0), to_screen * p2],
                Stroke::new(2.0, Color32::GRAY),
            );

            response.drag_stopped |= drag_stopped;
        }

        response
    }
}

pub struct EasingValue<'a> {
    value: &'a mut Easing,
    show_graph: bool,

    /// Easings in this vec will not be shown in the combobox
    disabled_easings: Vec<Easing>,
}

impl<'a> EasingValue<'a> {
    pub fn new(value: &'a mut Easing) -> Self {
        Self {
            value,
            show_graph: true,

            disabled_easings: vec![],
        }
    }

    pub fn show_graph(mut self, show_graph: bool) -> Self {
        self.show_graph = show_graph;
        self
    }

    #[allow(dead_code)]
    pub fn disabled_easings(mut self, disabled_easings: Vec<Easing>) -> Self {
        self.disabled_easings = disabled_easings;
        self
    }
}

/// Draw a easing curve with a [`Ui`] on the given [`Rect`]
pub fn draw_easing(
    ui: &mut Ui,
    rect: Rect,
    easing: Easing,
    reverse: bool,
    mirror: bool,
    color: Color32,
) {
    let painter = ui.painter_at(rect);
    let to_screen =
        emath::RectTransform::from_to(Rect::from_min_size(Pos2::ZERO, Vec2::new(1.0, 1.0)), rect);

    let points: Vec<_> = std::iter::repeat(0.0)
        .take(40)
        .enumerate()
        .map(|(i, _)| {
            let x = i as f32 / 40.0;
            if reverse {
                if mirror {
                    Pos2::new(1.0 - easing.ease(1.0 - x), x)
                } else {
                    Pos2::new(easing.ease(1.0 - x), x)
                }
            } else if mirror {
                Pos2::new(x, easing.ease(x))
            } else {
                Pos2::new(x, 1.0 - easing.ease(x))
            }
        })
        .map(|x| to_screen * x)
        .collect();

    painter.add(PathShape::line(points, Stroke::new(2.0, color)));
}

impl Widget for EasingValue<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            let mut combobox_changed = false;
            let mut response = egui::ComboBox::from_label("")
                .selected_text(format!("{}", self.value))
                .show_ui(ui, |ui| {
                    for easing in Easing::iter().filter(|x| !self.disabled_easings.contains(x)) {
                        if ui
                            .selectable_label(*self.value == easing, format!("{}", easing))
                            .clicked()
                        {
                            combobox_changed = true;
                            if let Easing::Custom(_, _, _, _) = easing {
                                *self.value = Easing::Custom(0.5, 0.0, 0.5, 1.0);
                            } else {
                                *self.value = easing;
                            }
                        }
                    }
                })
                .response;

            // temporary workaround for change handling: .change() is reserved by egui,
            // we use drag_stopped for change handling as the same as DragValue
            response.drag_stopped |= combobox_changed;

            if self.show_graph {
                let mut graph_response = ui.add_sized(
                    Vec2::new(ui.available_width(), ui.available_width() / 3.0 * 2.0),
                    EasingGraph::new(self.value),
                );

                graph_response.drag_stopped |= response.drag_stopped;

                graph_response
            } else {
                response
            }
        })
        .inner
    }
}
