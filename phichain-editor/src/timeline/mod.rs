pub mod click;
pub mod container;
pub mod drag_selection;
pub mod event;
pub mod kind;
pub mod note;
pub mod pending;
pub mod pending_status;
pub mod settings;
pub mod target;

use crate::constants::{BASE_ZOOM, INDICATOR_POSITION};
use crate::tab::timeline::TimelineViewport;
use crate::timeline::click::TimelineClickPlugin;
use crate::timeline::container::spawn_timeline_container;
use crate::timeline::drag_selection::TimelineDragSelectionPlugin;
use crate::timeline::event::{
    event_timeline_drag_selection, event_timeline_name, event_timeline_ui, EventTimeline,
};
use crate::timeline::kind::TimelineKindRegistrationExt;
use crate::timeline::note::{
    note_timeline_drag_selection, note_timeline_name, note_timeline_ui, NoteTimeline,
};
use crate::timeline::pending::PendingPlugin;
use crate::timeline::settings::TimelineSettings;
use crate::timing::ChartTime;
use bevy::app::{App, Plugin};
use bevy::ecs::system::SystemParam;
use bevy::math::Vec2;
use bevy::prelude::*;
use phichain_chart::beat;
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::BpmList;
use phichain_game::audio::AudioDuration;

/// Opacity of the vertical lane separator lines on timelines
pub const LANE_LINE_OPACITY: f32 = 40.0 / 255.0;

pub struct TimelinePlugin;

impl Plugin for TimelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TimelineDragSelectionPlugin,
            TimelineClickPlugin,
            PendingPlugin,
        ))
        .insert_resource(TimelineViewport(bevy::math::Rect::from_corners(
            Vec2::ZERO,
            Vec2::ZERO,
        )))
        .insert_resource(TimelineSettings::default())
        .add_systems(Startup, spawn_timeline_container)
        .add_timeline_kind(
            NoteTimeline,
            note_timeline_ui,
            note_timeline_drag_selection,
            note_timeline_name,
        )
        .add_timeline_kind(
            EventTimeline,
            event_timeline_ui,
            event_timeline_drag_selection,
            event_timeline_name,
        );
    }
}

/// Resources and context to work with timelines
#[derive(SystemParam)]
pub struct TimelineContext<'w> {
    bpm_list: Res<'w, BpmList>,
    pub settings: Res<'w, TimelineSettings>,
    current_time: Res<'w, ChartTime>,
    pub viewport: Res<'w, TimelineViewport>,

    audio_duration: Res<'w, AudioDuration>,
}

impl TimelineContext<'_> {
    /// Beat grid lines between two y positions within the audio, as `(index, time)`
    ///
    /// `index` counts `1 / density` beats from beat 0.
    pub fn beat_lines_between(
        &self,
        top: f32,
        bottom: f32,
    ) -> impl Iterator<Item = (u32, f32)> + '_ {
        let density = self.settings.density;
        let latest = self.y_to_time(top).min(self.audio_duration.0.as_secs_f32());
        let earliest = self.y_to_time(bottom);
        let first = (self.bpm_list.beat_at_f32(earliest) * density as f32)
            .ceil()
            .max(0.0) as u32;
        let last = (self.bpm_list.beat_at_f32(latest) * density as f32).floor();
        (first..)
            .take_while(move |index| *index as f32 <= last)
            .map(move |index| (index, self.bpm_list.time_at(beat!(0, index, density))))
    }

    pub fn time_to_y(&self, time: f32) -> f32 {
        (self.current_time.0 - time) * BASE_ZOOM * self.settings.zoom
            + self.viewport.0.min.y
            + self.viewport.0.height() * INDICATOR_POSITION
    }

    pub fn beat_to_y(&self, beat: Beat) -> f32 {
        self.time_to_y(self.bpm_list.time_at(beat))
    }

    pub fn y_to_time(&self, y: f32) -> f32 {
        self.current_time.0
            - (y - (self.viewport.0.min.y + self.viewport.0.height() * INDICATOR_POSITION))
                / (BASE_ZOOM * self.settings.zoom)
    }

    #[allow(dead_code)]
    pub fn y_to_beat(&self, y: f32) -> Beat {
        self.bpm_list.beat_at(self.y_to_time(y))
    }

    pub fn y_to_beat_f32(&self, y: f32) -> f32 {
        self.bpm_list.beat_at_f32(self.y_to_time(y))
    }
}

pub mod common {
    use crate::constants::INDICATOR_POSITION;
    use crate::tab::timeline::TimelineViewport;
    use crate::timeline::container::{
        insert_timeline, laid_out_timelines, move_separator, ordered_timelines, swap_timelines,
    };
    use crate::timeline::event::EventTimeline;
    use crate::timeline::kind::timeline_name;
    use crate::timeline::note::NoteTimeline;
    use crate::timeline::TimelineContext;
    use crate::utils::convert::BevyEguiConvert;
    use bevy::ecs::system::SystemState;
    use bevy::prelude::*;
    use egui::{
        Align, Align2, Color32, CornerRadius, FontId, Id, Layout, Rect, RichText, Sense, Ui,
    };
    use phichain_chart::bpm_list::BpmList;
    use phichain_chart::line::Line;

    /// Opacity of the horizontal beat grid lines on timelines
    const BEAT_LINE_OPACITY: f32 = 40.0 / 255.0;

    const BEAT_LABEL_SIZE: f32 = 11.0;

    /// The same type → target menu is used by badges and the empty state.
    fn creation_menu(ui: &mut Ui, world: &mut World, after: Option<Entity>) {
        for (label, note) in [
            (t!("tab.timeline.note_timeline"), true),
            (t!("tab.timeline.event_timeline"), false),
        ] {
            ui.menu_button(label, |ui| {
                let mut target = None;
                if ui
                    .button(RichText::new(t!("tab.timeline.binding")).strong())
                    .clicked()
                {
                    target = Some(None);
                }
                ui.separator();
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for (entity, line) in world.query::<(Entity, &Line)>().iter(world) {
                            ui.push_id(entity, |ui| {
                                if ui.button(&line.name).clicked() {
                                    target = Some(Some(entity));
                                }
                            });
                        }
                    });
                if let Some(target) = target {
                    if note {
                        insert_timeline(world, NoteTimeline, target, after);
                    } else {
                        insert_timeline(world, EventTimeline, target, after);
                    }
                    ui.close();
                    ui.ctx().request_repaint();
                }
            });
        }
    }

    pub fn empty_timeline_ui(ui: &mut Ui, world: &mut World) {
        let viewport = ui.clip_rect();
        let rect = Rect::from_min_max(
            egui::pos2(
                viewport.left(),
                (viewport.center().y - 50.0).max(viewport.top()),
            ),
            viewport.right_bottom(),
        );
        ui.scope_builder(
            egui::UiBuilder::new()
                .id_salt("empty_timeline")
                .max_rect(rect)
                .layout(Layout::top_down(Align::Center)),
            |ui| {
                ui.heading(t!("tab.timeline.empty_title"));
                ui.add_space(6.0);
                ui.label(RichText::new(t!("tab.timeline.empty_description")).weak());
                ui.add_space(12.0);
                ui.menu_button(
                    format!(
                        "{} {}",
                        egui_phosphor::regular::PLUS,
                        t!("tab.timeline.create")
                    ),
                    |ui| creation_menu(ui, world, None),
                );
            },
        );
    }

    pub fn timeline_badge_ui(ui: &mut Ui, world: &mut World, timeline: Entity, viewport: Rect) {
        let id = Id::new(("timeline_badge", timeline));
        let popup_id = id.with("add_menu");
        let name = timeline_name(world, timeline);
        let galley =
            ui.painter()
                .layout_no_wrap(name.clone(), Default::default(), Default::default());
        let badge = Rect::from_center_size(
            egui::pos2(
                viewport.center().x,
                viewport.top() + viewport.height() * 0.1,
            ),
            galley.size(),
        );
        let action_width = badge.height();
        let remove_rect = Rect::from_min_max(
            badge.left_top() - egui::vec2(action_width, 0.0),
            badge.left_bottom(),
        );
        let add_rect = Rect::from_min_max(
            badge.right_top(),
            badge.right_bottom() + egui::vec2(action_width, 0.0),
        );
        let expanded = remove_rect.union(add_rect);
        // A click on an action can also activate the underlying drag-selection surface.
        // Only a badge drag should hide these controls, including on the release frame.
        let dragging = [ui.ctx().dragged_id(), ui.ctx().drag_stopped_id()]
            .into_iter()
            .flatten()
            .any(|dragged| {
                ordered_timelines(world)
                    .into_iter()
                    .any(|timeline| dragged == Id::new(("timeline_badge", timeline)))
            });
        if dragging {
            egui::Popup::close_id(ui.ctx(), popup_id);
        }
        let show_actions = !dragging
            && (ui.rect_contains_pointer(expanded) || egui::Popup::is_id_open(ui.ctx(), popup_id));
        let visuals = ui.visuals().widgets.inactive;
        if show_actions {
            // One continuous background; only the outer corners are rounded.
            ui.painter()
                .rect_filled(expanded, visuals.corner_radius, visuals.weak_bg_fill);
        }
        ui.put(badge, |ui: &mut Ui| {
            ui.dnd_drag_source(id, timeline, |ui| {
                if !show_actions {
                    ui.painter().rect(
                        badge,
                        visuals.corner_radius,
                        visuals.weak_bg_fill,
                        visuals.bg_stroke,
                        egui::StrokeKind::Inside,
                    );
                }
                ui.add(
                    egui::Label::new(RichText::new(&name).color(Color32::WHITE))
                        .truncate()
                        .selectable(false),
                )
            })
            .response
        });

        if show_actions {
            let remove = badge_action(
                ui,
                remove_rect,
                id.with("remove"),
                egui_phosphor::regular::X,
                CornerRadius {
                    ne: 0,
                    se: 0,
                    ..visuals.corner_radius
                },
            )
            .on_hover_text(t!("tab.timeline.remove"));
            let add = badge_action(
                ui,
                add_rect,
                id.with("add"),
                egui_phosphor::regular::PLUS,
                CornerRadius {
                    nw: 0,
                    sw: 0,
                    ..visuals.corner_radius
                },
            )
            .on_hover_text(t!("tab.timeline.add"));
            ui.painter().rect_stroke(
                expanded,
                visuals.corner_radius,
                visuals.bg_stroke,
                egui::StrokeKind::Inside,
            );
            for x in [badge.left(), badge.right()] {
                ui.painter()
                    .vline(x, badge.y_range(), (1.0, ui.visuals().extreme_bg_color));
            }
            egui::Popup::menu(&add)
                .id(popup_id)
                .show(|ui| creation_menu(ui, world, Some(timeline)));
            if remove.clicked() {
                world.despawn(timeline);
                egui::Popup::close_id(ui.ctx(), popup_id);
                ui.ctx().request_repaint();
                return;
            }
        }

        let response = ui.allocate_rect(viewport, Sense::hover());
        if let Some(payload) = response.dnd_hover_payload::<Entity>() {
            if *payload != timeline {
                ui.painter().rect_filled(
                    viewport,
                    0.0,
                    egui_dock::Style::default().overlay.selection_color,
                );
            }
        }
        if let Some(payload) = response.dnd_release_payload::<Entity>() {
            swap_timelines(world, *payload, timeline);
        }
    }

    /// Match the existing badge's height and theme instead of imposing button padding.
    fn badge_action(
        ui: &mut Ui,
        rect: Rect,
        id: Id,
        icon: &str,
        corner_radius: CornerRadius,
    ) -> egui::Response {
        let response = ui.interact(rect, id, Sense::click());
        let visuals = ui.style().interact(&response);
        ui.painter()
            .rect_filled(rect, corner_radius, visuals.weak_bg_fill);
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            icon,
            Default::default(),
            visuals.text_color(),
        );
        response
    }

    pub fn beat_line_ui(ui: &mut Ui, world: &mut World) {
        let mut state: SystemState<TimelineContext> = SystemState::new(world);
        let ctx = state.get_mut(world);
        let denom = ctx.settings.density;
        // labels sit above their lines, so lines just below the viewport can still show one
        let lines =
            ctx.beat_lines_between(ctx.viewport.0.min.y, ctx.viewport.0.max.y + BEAT_LABEL_SIZE);
        for (numer, beat_time) in lines {
            let primary = numer.is_multiple_of(denom);
            let rect = egui::Rect::from_center_size(
                egui::Pos2::new(
                    ctx.viewport.0.width() / 2.0 + ctx.viewport.0.min.x,
                    ctx.time_to_y(beat_time),
                ),
                egui::Vec2::new(ctx.viewport.0.width(), if primary { 2.0 } else { 0.5 }),
            );
            ui.painter()
                .rect_filled(rect, 0.0, Color32::WHITE.gamma_multiply(BEAT_LINE_OPACITY));
            if !primary {
                continue;
            }
            ui.painter().text(
                rect.left_top() + egui::Vec2::new(4.0, 0.0),
                Align2::LEFT_BOTTOM,
                numer / denom,
                FontId::monospace(BEAT_LABEL_SIZE),
                Color32::WHITE,
            );
        }
    }

    /// Highlight the beat line under the cursor and show its full beat
    pub fn cursor_beat_ui(ui: &mut Ui, world: &mut World) {
        let mut state: SystemState<TimelineContext> = SystemState::new(world);
        let ctx = state.get_mut(world);
        let viewport = ctx.viewport.0.into_egui();
        let Some(pos) = ui.ctx().pointer_hover_pos() else {
            return;
        };
        if !ui.rect_contains_pointer(viewport) {
            return;
        }

        let beat = ctx.settings.attach(ctx.y_to_beat_f32(pos.y));
        let y = ctx.beat_to_y(beat);
        let color = ui.visuals().selection.bg_fill;
        ui.painter().hline(viewport.x_range(), y, (1.0, color));

        let galley = ui.painter().layout_no_wrap(
            format!("{beat:?}"),
            FontId::monospace(BEAT_LABEL_SIZE),
            Color32::WHITE,
        );
        let rect = Align2::LEFT_BOTTOM
            .anchor_size(egui::Pos2::new(viewport.left() + 4.0, y), galley.size());
        ui.painter().rect_filled(rect.expand(2.0), 2.0, color);
        ui.painter().galley(rect.min, galley, Color32::WHITE);
    }

    pub fn separator_ui(ui: &mut Ui, world: &mut World) {
        let viewport = world.resource::<TimelineViewport>().0.into_egui();
        let timelines = laid_out_timelines(world);

        let separators = timelines.len().saturating_sub(1);
        for (index, item) in timelines.iter().enumerate().take(separators) {
            let rect = egui::Rect::from_center_size(
                egui::Pos2::new(item.viewport.right(), viewport.center().y),
                egui::Vec2::new(2.0, viewport.height()),
            );
            ui.painter().rect_filled(rect, 0.0, Color32::WHITE);

            let response = ui
                .allocate_rect(rect, Sense::drag())
                .on_hover_cursor(egui::CursorIcon::ResizeHorizontal);
            if response.dragged() {
                let delta_x = response.drag_delta().x;
                let delta_percent = delta_x / viewport.width();
                move_separator(world, index, delta_percent);
            }
        }
    }

    pub fn indicator_ui(ui: &mut Ui, world: &mut World) {
        let mut state: SystemState<TimelineContext> = SystemState::new(world);
        let ctx = state.get_mut(world);
        ui.painter().rect_filled(
            egui::Rect::from_center_size(
                egui::Pos2::new(
                    ctx.viewport.0.center().x,
                    ctx.viewport.0.min.y + ctx.viewport.0.height() * INDICATOR_POSITION,
                ),
                egui::Vec2::new(ctx.viewport.0.width(), 2.0),
            ),
            0.0,
            Color32::WHITE,
        );
    }

    pub fn bpm_change_ui(ui: &mut Ui, world: &mut World) {
        let mut state: SystemState<(TimelineContext, Res<BpmList>)> = SystemState::new(world);
        let (ctx, bpm_list) = state.get_mut(world);
        for point in &bpm_list.0 {
            let y = ctx.time_to_y(bpm_list.time_at(point.beat));

            let rect = egui::Rect::from_center_size(
                egui::Pos2::new(ctx.viewport.0.width() / 2.0 + ctx.viewport.0.min.x, y),
                egui::Vec2::new(ctx.viewport.0.width(), 0.5),
            );

            ui.painter().text(
                rect.center_top(),
                Align2::CENTER_BOTTOM,
                format!("BPM: {}", point.bpm),
                FontId::monospace(14.0),
                Color32::WHITE,
            );
        }
    }
}
