use egui::{CentralPanel, Color32, Context, CornerRadius, Frame, Margin, Response, Stroke};
use egui_dock::{DockArea, DockState, Style, TabViewer};

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

pub(super) fn dock_style(egui_style: &egui::Style) -> Style {
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

    for tab in [
        &mut style.tab.active,
        &mut style.tab.focused,
        &mut style.tab.inactive,
        &mut style.tab.hovered,
        &mut style.tab.active_with_kb_focus,
        &mut style.tab.focused_with_kb_focus,
        &mut style.tab.inactive_with_kb_focus,
    ] {
        tab.corner_radius = CornerRadius::ZERO;
        tab.outline_color = Color32::TRANSPARENT;
        tab.bg_fill = background;
        tab.text_color = visuals.text_color();
    }
    for tab in [
        &mut style.tab.active,
        &mut style.tab.focused,
        &mut style.tab.active_with_kb_focus,
        &mut style.tab.focused_with_kb_focus,
    ] {
        tab.bg_fill = selected;
        tab.text_color = visuals.strong_text_color();
    }
    style.tab.hovered.bg_fill = selected;
    style.tab.hovered.text_color = visuals.strong_text_color();
    for tab in [
        &mut style.tab.active_with_kb_focus,
        &mut style.tab.focused_with_kb_focus,
        &mut style.tab.inactive_with_kb_focus,
    ] {
        tab.outline_color = visuals.selection.stroke.color;
    }

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
