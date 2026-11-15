use egui::{CentralPanel, Color32, Context, CornerRadius, Frame, Margin, Response, Stroke};
use egui_dock::{DockArea, DockState, Style, TabInteractionStyle, TabViewer};

pub(super) fn show_dock_area<Tab>(
    ctx: &Context,
    state: &mut DockState<Tab>,
    viewer: &mut impl TabViewer<Tab = Tab>,
) {
    // Capture content padding before removing the floating window's outer padding.
    let style = dock_style(&ctx.style());
    CentralPanel::default()
        .frame(
            Frame::central_panel(&ctx.style())
                .inner_margin(0)
                .fill(Color32::TRANSPARENT),
        )
        .show(ctx, |ui| {
            // egui_dock builds its window frames from this Ui's style. Keep the
            // override local so dialogs and tab contents retain their own spacing.
            ui.style_mut().spacing.window_margin = Margin::ZERO;
            // Edge-to-edge dock contents are rectangular, including at the corners.
            ui.style_mut().visuals.window_corner_radius = CornerRadius::ZERO;
            DockArea::new(state)
                .show_leaf_collapse_buttons(false)
                .style(style)
                .show_inside(ui, viewer);
        });
}

fn dock_style(egui_style: &egui::Style) -> Style {
    let mut style = Style::from_egui(egui_style);
    let visuals = &egui_style.visuals;
    let background = visuals.window_fill();
    let selected = background.lerp_to_gamma(visuals.widgets.inactive.bg_fill, 0.3);

    // One continuous header, with only the dock separator between adjacent panes.
    style.tab_bar.bg_fill = background;
    style.tab_bar.corner_radius = CornerRadius::ZERO;
    style.tab_bar.height = 24.0;
    style.tab_bar.inner_margin = Margin::ZERO;
    style.tab_bar.hline_color =
        background.lerp_to_gamma(visuals.widgets.noninteractive.bg_stroke.color, 0.6);
    style.tab.tab_body.stroke = Stroke::NONE;
    style.tab.tab_body.corner_radius = CornerRadius::ZERO;
    style.tab.spacing = 0.0;
    // Size short tabs to their labels while keeping the native close-button target.
    style.tab.minimum_width = None;
    style.tab.hline_below_active_tab_name = true;

    let inactive = TabInteractionStyle {
        corner_radius: CornerRadius::ZERO,
        outline_color: Color32::TRANSPARENT,
        bg_fill: background,
        text_color: visuals.text_color(),
    };
    let active = TabInteractionStyle {
        bg_fill: selected,
        text_color: visuals.strong_text_color(),
        ..inactive.clone()
    };
    style.tab.inactive = inactive.clone();
    style.tab.active = active.clone();
    style.tab.focused = active.clone();
    style.tab.hovered = active.clone();
    style.tab.inactive_with_kb_focus = TabInteractionStyle {
        outline_color: visuals.selection.stroke.color,
        ..inactive
    };
    style.tab.active_with_kb_focus = TabInteractionStyle {
        outline_color: visuals.selection.stroke.color,
        ..active
    };
    style.tab.focused_with_kb_focus = style.tab.active_with_kb_focus.clone();

    style.buttons.close_tab_color = visuals.weak_text_color();
    style.buttons.close_all_tabs_color = visuals.weak_text_color();
    style.buttons.close_all_tabs_border_color = Color32::TRANSPARENT;
    style
}

pub(super) fn paint_active_tab(response: &Response) {
    let style = response.ctx.style();
    let visuals = &style.visuals;
    let accent = visuals
        .selection
        .bg_fill
        .lerp_to_gamma(visuals.selection.stroke.color, 0.35);
    let rect = response.rect;
    response
        .ctx
        .layer_painter(response.layer_id)
        .with_clip_rect(response.interact_rect)
        .hline(
            rect.x_range(),
            rect.bottom() - 1.0,
            Stroke::new(1.0_f32, accent),
        );
}
