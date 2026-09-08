use crate::ui::edit::{EditResponse, EditWidget};
use egui::epaint::PathShape;
use egui::{
    emath, Color32, Grid, Label, Layout, Pos2, Rect, Response, RichText, Sense, Stroke, Ui, Vec2,
    Widget,
};
use phichain_chart::easing::Easing;
use strum::IntoEnumIterator;

const CURVE_SAMPLES: usize = 100;

pub struct EasingGraph<'a> {
    value: &'a mut Easing,
    inverse: bool,
    mirror: bool,
    color: Color32,
    rect: Option<Rect>,
}

impl<'a> EasingGraph<'a> {
    pub fn new(value: &'a mut Easing) -> Self {
        Self {
            value,
            inverse: false,
            mirror: false,
            color: Color32::WHITE,
            rect: None,
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

impl EasingGraph<'_> {
    /// Place the curve on an existing canvas instead of allocating an inspector graph.
    pub fn rect(mut self, rect: Rect) -> Self {
        self.rect = Some(rect);
        self
    }

    fn show(self, ui: &mut Ui) -> Vec<Response> {
        let graph = if let Some(rect) = self.rect {
            ui.interact(rect, ui.id().with("graph"), Sense::hover())
        } else {
            ui.allocate_response(
                Vec2::new(ui.available_width(), ui.available_width() * 2.0 / 3.0),
                Sense::hover(),
            )
        };
        let mut responses = vec![graph.clone()];
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, Vec2::splat(1.0)),
            graph.rect,
        );
        if let Easing::Custom { x1, y1, x2, y2 } = self.value {
            for (index, (x, y)) in [(x1, y1), (x2, y2)].into_iter().enumerate() {
                let mut point = Pos2::new(*x, *y);
                let screen = to_screen * graph_point(point, self.inverse, self.mirror);
                let mut handle = ui.interact(
                    Rect::from_center_size(screen, Vec2::splat(8.0)),
                    ui.id().with(("control", index)),
                    Sense::drag(),
                );
                if handle.dragged() || handle.drag_stopped() {
                    let delta = if handle.drag_stopped() {
                        ui.input(|input| input.pointer.delta())
                    } else {
                        handle.drag_delta()
                    };
                    if delta != Vec2::ZERO {
                        let display = graph_point(point, self.inverse, self.mirror)
                            + delta / graph.rect.size().max(Vec2::splat(1.0));
                        point = curve_point(display, self.inverse, self.mirror);
                        point.x = point.x.clamp(0.0, 1.0);
                        point.y = point.y.clamp(0.0, 1.0);
                    }
                    if point != Pos2::new(*x, *y) {
                        *x = point.x;
                        *y = point.y;
                        handle.mark_changed();
                    }
                }
                responses.push(handle);
                let screen = to_screen * graph_point(point, self.inverse, self.mirror);
                ui.painter()
                    .circle(screen, 4.0, Color32::WHITE, Stroke::NONE);
                let anchor = Pos2::new(index as f32, index as f32);
                ui.painter().line_segment(
                    [
                        to_screen * graph_point(anchor, self.inverse, self.mirror),
                        screen,
                    ],
                    Stroke::new(2.0_f32, Color32::GRAY),
                );
            }
        }
        draw_easing(
            ui,
            graph.rect,
            *self.value,
            self.inverse,
            self.mirror,
            self.color,
        );
        // Coordinate inputs belong below the graph, including when it overlays a timeline.
        if let Easing::Custom { x1, y1, x2, y2 } = self.value {
            let rect = Rect::from_min_size(
                graph.rect.left_bottom() + Vec2::new(0.0, 4.0),
                Vec2::new(graph.rect.width(), ui.spacing().interact_size.y),
            );
            ui.scope_builder(
                egui::UiBuilder::new().id_salt("coordinates").max_rect(rect),
                |ui| {
                    ui.horizontal(|ui| {
                        for value in [x1, y1, x2, y2] {
                            responses.push(ui.add(egui::DragValue::new(value).speed(0.01)));
                        }
                    });
                },
            );
        }
        responses
    }
}

impl Widget for EasingGraph<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
            .into_iter()
            .reduce(|response, other| response.union(other))
            .unwrap()
    }
}

impl EditWidget for EasingGraph<'_> {
    fn edit_ui(self, ui: &mut Ui) -> EditResponse {
        EditResponse::Gesture(self.show(ui))
    }
}

fn graph_point(point: Pos2, inverse: bool, mirror: bool) -> Pos2 {
    if inverse {
        Pos2::new(if mirror { 1.0 - point.y } else { point.y }, 1.0 - point.x)
    } else {
        Pos2::new(point.x, if mirror { point.y } else { 1.0 - point.y })
    }
}

fn curve_point(point: Pos2, inverse: bool, mirror: bool) -> Pos2 {
    if inverse {
        Pos2::new(1.0 - point.y, if mirror { 1.0 - point.x } else { point.x })
    } else {
        Pos2::new(point.x, if mirror { point.y } else { 1.0 - point.y })
    }
}

pub struct EasingValue<'a> {
    value: &'a mut Easing,

    /// Easings in this vec will not be shown in the combobox
    disabled_easings: Vec<Easing>,
}

impl<'a> EasingValue<'a> {
    pub fn new(value: &'a mut Easing) -> Self {
        Self {
            value,

            disabled_easings: vec![],
        }
    }

    #[allow(dead_code)]
    pub fn disabled_easings(mut self, disabled_easings: Vec<Easing>) -> Self {
        self.disabled_easings = disabled_easings;
        self
    }
}

/// Draw an easing curve with a [`Ui`] on the given [`Rect`]
///
/// Curves of overshooting easing functions (Back / Elastic) may overflow the given [`Rect`], but will not affect layout
pub fn draw_easing(
    ui: &mut Ui,
    rect: Rect,
    easing: Easing,
    reverse: bool,
    mirror: bool,
    color: Color32,
) {
    let rect = rect.expand2(rect.size());
    let painter = ui.painter_at(rect);
    let to_screen =
        emath::RectTransform::from_to(Rect::from_min_size(Pos2::ZERO, Vec2::new(1.0, 1.0)), rect);

    let points: Vec<_> = std::iter::repeat_n(0.0, CURVE_SAMPLES)
        .enumerate()
        .map(|(i, _)| {
            let x = i as f32 / CURVE_SAMPLES as f32;
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
        .map(|x| x / 3.0 + Vec2::splat(1.0 / 3.0))
        .map(|x| to_screen * x)
        .collect();

    painter.add(PathShape::line(points, Stroke::new(2.0_f32, color)));
}

fn draw_easing_options(ui: &mut Ui, easing: Easing, selected: bool, name: &str) -> Response {
    crate::ui::widgets::button_frame::button_frame(ui, selected, |ui, text_color| {
        ui.vertical(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(80.0, 40.0), Sense::hover());
            draw_easing(ui, rect, easing, false, false, Color32::DARK_GRAY);
            ui.put(
                rect,
                Label::new(RichText::new(name).color(text_color)).selectable(false),
            );
        });
    })
}

impl Widget for EasingValue<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut combobox_changed = false;
        ui.horizontal(|ui| {
            let mut response = egui::ComboBox::from_label("")
                .height(300.0)
                .selected_text(format!("{}", self.value))
                .show_ui(ui, |ui| {
                    ui.scope(|ui| {
                        ui.columns_const(|[linear, custom]| {
                            linear.with_layout(
                                Layout::top_down_justified(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .selectable_label(self.value.is_linear(), "Linear")
                                        .clicked()
                                    {
                                        combobox_changed = true;
                                        *self.value = Easing::Linear;
                                    }
                                },
                            );
                            custom.with_layout(
                                Layout::top_down_justified(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .selectable_label(self.value.is_custom(), "Custom")
                                        .clicked()
                                    {
                                        combobox_changed = true;
                                        *self.value = Easing::Custom {
                                            x1: 0.5,
                                            y1: 0.0,
                                            x2: 0.5,
                                            y2: 1.0,
                                        };
                                    }
                                },
                            );
                        });
                        Grid::new("easing-grid").num_columns(3).show(ui, |ui| {
                            for easing in Easing::iter().filter(|x| {
                                !self.disabled_easings.contains(x)
                                    && !x.is_custom()
                                    && !x.is_linear()
                                    && !x.is_steps()
                                    && !x.is_elastic()
                            }) {
                                let selected = self.value == &easing;

                                let response = draw_easing_options(
                                    ui,
                                    easing,
                                    selected,
                                    format!("{easing}").trim_start_matches("Ease"),
                                );

                                if response.clicked() {
                                    combobox_changed = true;
                                    *self.value = easing;
                                }

                                if easing.is_in_out() {
                                    ui.end_row();
                                }
                            }

                            if draw_easing_options(
                                ui,
                                Easing::Steps { count: 4 },
                                self.value.is_steps(),
                                "Steps",
                            )
                            .clicked()
                            {
                                combobox_changed = true;
                                *self.value = Easing::Steps { count: 4 };
                            }

                            if draw_easing_options(
                                ui,
                                Easing::Elastic { omega: 20.0 },
                                self.value.is_elastic(),
                                "Elastic",
                            )
                            .clicked()
                            {
                                combobox_changed = true;
                                *self.value = Easing::Elastic { omega: 20.0 };
                            }
                        });
                    });
                })
                .response;

            if combobox_changed {
                response.mark_changed();
            }

            response
        })
        .inner
    }
}

impl EditWidget for EasingValue<'_> {
    fn edit_ui(self, ui: &mut Ui) -> EditResponse {
        EditResponse::Once(self.ui(ui))
    }
}

/// Continuous parameter of a Steps or Elastic easing, separate from type selection.
pub struct EasingParameter<'a>(pub &'a mut Easing);

impl Widget for EasingParameter<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        match self.0 {
            Easing::Steps { count } => ui.add(egui::DragValue::new(count).speed(1).range(1..=64)),
            Easing::Elastic { omega } => {
                ui.add(egui::DragValue::new(omega).speed(0.1).range(10.0..=128.0))
            }
            _ => ui.allocate_response(Vec2::ZERO, Sense::hover()),
        }
    }
}

impl EditWidget for EasingParameter<'_> {
    fn edit_ui(self, ui: &mut Ui) -> EditResponse {
        EditResponse::Gesture(vec![self.ui(ui)])
    }
}
