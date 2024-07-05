pub mod drag_selection;
pub mod event;
pub mod note;
pub mod settings;

use crate::audio::AudioDuration;
use crate::constants::{BASE_ZOOM, INDICATOR_POSITION};
use crate::tab::timeline::TimelineViewport;
use crate::timeline::event::EventTimeline;
use crate::timeline::note::NoteTimeline;
use crate::timeline::settings::TimelineSettings;
use crate::timing::ChartTime;
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Entity, Res, ResMut, World};
use egui::{Rect, Ui};
use enum_dispatch::enum_dispatch;
use phichain_chart::beat;
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::BpmList;

// TODO: make all resources mutable
/// Resources and context to work with timelines
///
/// This [`SystemParam`] conflicts with all mutable resources it contains (https://bevyengine.org/learn/errors/b0002/):
///
/// - [`TimelineSettings`]
///
/// So it is impossible to have both [`TimelineContext`] and [`Res<TimelineSettings>`] (or [`ResMut<TimelineSettings>`]) params of a system
///
/// Instead, access the required resources directly from [`TimelineContext`]: `ctx.timeline_settings`
#[derive(SystemParam)]
pub struct TimelineContext<'w> {
    bpm_list: Res<'w, BpmList>,
    pub timeline_settings: ResMut<'w, TimelineSettings>,
    current_time: Res<'w, ChartTime>,
    pub viewport: Res<'w, TimelineViewport>,

    audio_duration: Res<'w, AudioDuration>,
}

impl<'w> TimelineContext<'w> {
    pub fn primary_beat_times(&self) -> Vec<f32> {
        std::iter::repeat(0)
            .enumerate()
            .map(|(i, _)| self.bpm_list.time_at(Beat::from(i as f32)))
            .take_while(|x| x <= &self.audio_duration.0.as_secs_f32())
            .collect()
    }

    pub fn secondary_beat_times(&self) -> Vec<f32> {
        std::iter::repeat(0)
            .enumerate()
            .map(|(i, _)| {
                self.bpm_list
                    .time_at(beat!(0, i, self.timeline_settings.density))
            })
            .take_while(|x| x <= &self.audio_duration.0.as_secs_f32())
            .collect()
    }

    pub fn time_to_y(&self, time: f32) -> f32 {
        (self.current_time.0 - time) * BASE_ZOOM * self.timeline_settings.zoom
            + self.viewport.0.min.y
            + self.viewport.0.height() * INDICATOR_POSITION
    }

    pub fn beat_to_y(&self, beat: Beat) -> f32 {
        self.time_to_y(self.bpm_list.time_at(beat))
    }

    pub fn y_to_time(&self, y: f32) -> f32 {
        self.current_time.0
            - (y - (self.viewport.0.min.y + self.viewport.0.height() * INDICATOR_POSITION))
                / (BASE_ZOOM * self.timeline_settings.zoom)
    }

    pub fn y_to_beat(&self, y: f32) -> Beat {
        self.bpm_list.beat_at(self.y_to_time(y))
    }
}

#[enum_dispatch(TimelineItem)]
pub trait Timeline {
    fn ui(&self, ui: &mut Ui, world: &mut World, viewport: Rect);

    /// Handle drag selection on the timeline
    ///
    /// The selection param will be a rect where x represents the x value and y represents the time
    ///
    /// The selection will be cropped to fit the timeline, which means the x of the left-top corner of the timeline will be 0
    ///
    /// The return value of this function will be a vector contains all entities that are selected
    fn on_drag_selection(&self, world: &mut World, viewport: Rect, selection: Rect) -> Vec<Entity>;
}

#[enum_dispatch]
#[derive(Debug, Clone)]
pub enum TimelineItem {
    Note(NoteTimeline),
    Event(EventTimeline),
}

pub mod common {
    use crate::constants::INDICATOR_POSITION;
    use crate::timeline::TimelineContext;
    use bevy::ecs::system::SystemState;
    use bevy::prelude::*;
    use egui::{Align2, Color32, FontId, Sense, Ui};

    pub fn beat_line_ui(ui: &mut Ui, world: &mut World) {
        let mut state: SystemState<TimelineContext> = SystemState::new(world);
        let ctx = state.get_mut(world);
        for (index, beat_time) in ctx.primary_beat_times().iter().enumerate() {
            let rect = egui::Rect::from_center_size(
                egui::Pos2::new(
                    ctx.viewport.0.width() / 2.0 + ctx.viewport.0.min.x,
                    ctx.time_to_y(*beat_time),
                ),
                egui::Vec2::new(ctx.viewport.0.width(), 2.0),
            );
            ui.painter().rect_filled(
                rect,
                0.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, 40),
            );
            ui.painter().text(
                rect.left_top() + egui::Vec2::new(4.0, 0.0),
                Align2::LEFT_BOTTOM,
                index,
                FontId::monospace(14.0),
                Color32::WHITE,
            );
        }

        for (index, beat_time) in ctx.secondary_beat_times().iter().enumerate() {
            if index as u32 % ctx.timeline_settings.density == 0 {
                continue;
            }
            let rect = egui::Rect::from_center_size(
                egui::Pos2::new(
                    ctx.viewport.0.width() / 2.0 + ctx.viewport.0.min.x,
                    ctx.time_to_y(*beat_time),
                ),
                egui::Vec2::new(ctx.viewport.0.width(), 0.5),
            );
            ui.painter().rect_filled(
                rect,
                0.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, 40),
            );
            let numer = index as u32;
            let denom = ctx.timeline_settings.density;
            ui.painter().text(
                rect.left_top() + egui::Vec2::new(4.0, 0.0),
                Align2::LEFT_BOTTOM,
                format!("{}:{}/{}", numer / denom, numer % denom, denom),
                FontId::monospace(8.0),
                Color32::WHITE,
            );
        }
    }

    pub fn separator_ui(ui: &mut Ui, world: &mut World) {
        let mut state: SystemState<TimelineContext> = SystemState::new(world);
        let mut ctx = state.get_mut(world);

        for percent in ctx
            .timeline_settings
            .timelines
            .iter_mut()
            .map(|(_, percent)| percent)
        {
            let rect = egui::Rect::from_center_size(
                egui::Pos2::new(
                    ctx.viewport.0.min.x + *percent * ctx.viewport.0.width(),
                    ctx.viewport.0.center().y,
                ),
                egui::Vec2::new(2.0, ctx.viewport.0.height()),
            );
            ui.painter().rect_filled(rect, 0.0, Color32::WHITE);

            let response = ui
                .allocate_rect(rect, Sense::drag())
                .on_hover_cursor(egui::CursorIcon::ResizeHorizontal);
            if response.dragged() {
                let delta_x = response.drag_delta().x;
                let delta_percent = delta_x / ctx.viewport.0.width();
                // TODO: handle clamp range
                *percent += delta_percent;
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
}
