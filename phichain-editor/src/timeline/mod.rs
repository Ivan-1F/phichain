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
    use crate::timeline::container::{laid_out_timelines, move_separator, swap_timelines};
    use crate::timeline::kind::timeline_name;
    use crate::timeline::TimelineContext;
    use crate::utils::convert::BevyEguiConvert;
    use bevy::ecs::system::SystemState;
    use bevy::prelude::*;
    use egui::{epaint, Align2, Color32, FontId, Id, Sense, Ui};
    use phichain_chart::bpm_list::BpmList;

    /// Opacity of the horizontal beat grid lines on timelines
    const BEAT_LINE_OPACITY: f32 = 40.0 / 255.0;

    const PRIMARY_BEAT_LABEL_SIZE: f32 = 14.0;

    pub fn timeline_badge_ui(
        ui: &mut Ui,
        world: &mut World,
        timeline: Entity,
        viewport: egui::Rect,
    ) {
        let mut swap = None;

        let name = timeline_name(world, timeline);
        let galley =
            ui.painter()
                .layout_no_wrap(name.clone(), Default::default(), Default::default());

        let badge_rect = egui::Rect::from_center_size(
            egui::Pos2::new(
                viewport.center().x,
                viewport.top() + viewport.height() * 0.1,
            ),
            galley.size(),
        );

        let visuals = ui.style().visuals.widgets.inactive;

        ui.put(badge_rect, |ui: &mut Ui| {
            ui.dnd_drag_source(Id::new(("timeline_badge", timeline)), timeline, |ui| {
                ui.painter().rect(
                    badge_rect,
                    visuals.corner_radius,
                    visuals.weak_bg_fill,
                    visuals.bg_stroke,
                    epaint::StrokeKind::Inside,
                );
                ui.add(
                    egui::Label::new(egui::RichText::new(&name).color(Color32::WHITE))
                        .truncate()
                        .selectable(false),
                )
            })
            .response
        });

        // dropzone
        let response = ui.allocate_rect(viewport, Sense::hover());
        if let Some(dragged_payload) = response.dnd_release_payload::<Entity>() {
            swap = Some((*dragged_payload, timeline));
        }

        if let Some(hovered_payload) = response.dnd_hover_payload::<Entity>() {
            if *hovered_payload != timeline {
                ui.painter().rect_filled(
                    viewport,
                    0.0,
                    egui_dock::Style::default().overlay.selection_color,
                );
            }
        }

        if let Some((from, to)) = swap {
            swap_timelines(world, from, to);
        }
    }

    pub fn beat_line_ui(ui: &mut Ui, world: &mut World) {
        let mut state: SystemState<TimelineContext> = SystemState::new(world);
        let ctx = state.get_mut(world);
        let denom = ctx.settings.density;
        // labels sit above their lines, so lines just below the viewport can still show one
        let lines = ctx.beat_lines_between(
            ctx.viewport.0.min.y,
            ctx.viewport.0.max.y + PRIMARY_BEAT_LABEL_SIZE,
        );
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
            let (label, size) = if primary {
                ((numer / denom).to_string(), PRIMARY_BEAT_LABEL_SIZE)
            } else {
                (
                    format!("{}:{}/{}", numer / denom, numer % denom, denom),
                    8.0,
                )
            };
            ui.painter().text(
                rect.left_top() + egui::Vec2::new(4.0, 0.0),
                Align2::LEFT_BOTTOM,
                label,
                FontId::monospace(size),
                Color32::WHITE,
            );
        }
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
