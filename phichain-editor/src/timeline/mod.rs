pub mod container;
pub mod drag_selection;
pub mod event;
pub mod note;
pub mod settings;
pub mod target;

use crate::constants::{BASE_ZOOM, INDICATOR_POSITION};
use crate::tab::timeline::TimelineViewport;
use crate::timeline::container::spawn_timeline_container;
use crate::timeline::drag_selection::TimelineDragSelectionPlugin;
use crate::timeline::event::EventTimeline;
use crate::timeline::note::NoteTimeline;
use crate::timeline::settings::TimelineSettings;
use crate::timing::ChartTime;
use bevy::app::{App, Plugin};
use bevy::ecs::system::SystemParam;
use bevy::math::Vec2;
use bevy::prelude::*;
use egui::{Rect, Ui};
use phichain_chart::beat;
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::BpmList;
use phichain_game::audio::AudioDuration;

/// Opacity of the vertical lane separator lines on timelines
pub const LANE_LINE_OPACITY: f32 = 40.0 / 255.0;

pub struct TimelinePlugin;

impl Plugin for TimelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TimelineDragSelectionPlugin)
            .insert_resource(TimelineViewport(bevy::math::Rect::from_corners(
                Vec2::ZERO,
                Vec2::ZERO,
            )))
            .insert_resource(TimelineSettings::default())
            .add_systems(Startup, spawn_timeline_container);
    }
}

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
    pub settings: ResMut<'w, TimelineSettings>,
    current_time: Res<'w, ChartTime>,
    pub viewport: Res<'w, TimelineViewport>,

    audio_duration: Res<'w, AudioDuration>,
}

impl TimelineContext<'_> {
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
            .map(|(i, _)| self.bpm_list.time_at(beat!(0, i, self.settings.density)))
            .take_while(|x| x <= &self.audio_duration.0.as_secs_f32())
            .collect()
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

/// Behavior of a kind of timeline, implemented by its marker component
pub trait TimelineKind {
    fn ui(&self, ui: &mut Ui, world: &mut World, line: Entity, viewport: Rect);

    /// Handle drag selection on the timeline
    ///
    /// The selection param will be a rect where x represents the x value and y represents the time
    ///
    /// The selection will be cropped to fit the timeline, which means the x of the left-top corner of the timeline will be 0
    ///
    /// The return value of this function will be a vector contains all entities that are selected
    fn on_drag_selection(
        &self,
        world: &mut World,
        line: Entity,
        viewport: Rect,
        selection: Rect,
    ) -> Vec<Entity>;

    fn name(&self, world: &World, timeline: Entity) -> String;
}

/// The [`TimelineKind`] of the given timeline
pub fn timeline_kind(world: &World, timeline: Entity) -> Option<&'static dyn TimelineKind> {
    let timeline = world.get_entity(timeline).ok()?;
    if timeline.contains::<NoteTimeline>() {
        Some(&NoteTimeline)
    } else if timeline.contains::<EventTimeline>() {
        Some(&EventTimeline)
    } else {
        None
    }
}

pub mod common {
    use crate::constants::INDICATOR_POSITION;
    use crate::tab::timeline::TimelineViewport;
    use crate::timeline::container::{allocated_timelines, move_separator, swap_timelines};
    use crate::timeline::{timeline_kind, TimelineContext};
    use crate::utils::convert::BevyEguiConvert;
    use bevy::ecs::system::SystemState;
    use bevy::prelude::*;
    use egui::{epaint, Align2, Color32, FontId, Id, Sense, Ui};
    use phichain_chart::bpm_list::BpmList;

    /// Opacity of the horizontal beat grid lines on timelines
    const BEAT_LINE_OPACITY: f32 = 40.0 / 255.0;

    pub fn timeline_badge_ui(
        ui: &mut Ui,
        world: &mut World,
        timeline: Entity,
        viewport: egui::Rect,
        index: usize,
    ) {
        let mut swap = None;

        let Some(name) = timeline_kind(world, timeline).map(|kind| kind.name(world, timeline))
        else {
            return;
        };
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
            ui.dnd_drag_source(Id::new(&name).with(index), index, |ui| {
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
        if let Some(dragged_payload) = response.dnd_release_payload::<usize>() {
            swap = Some((*dragged_payload, index));
        }

        if let Some(hovered_payload) = response.dnd_hover_payload::<usize>() {
            if *hovered_payload != index {
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
        for (index, beat_time) in ctx.primary_beat_times().iter().enumerate() {
            let rect = egui::Rect::from_center_size(
                egui::Pos2::new(
                    ctx.viewport.0.width() / 2.0 + ctx.viewport.0.min.x,
                    ctx.time_to_y(*beat_time),
                ),
                egui::Vec2::new(ctx.viewport.0.width(), 2.0),
            );
            ui.painter()
                .rect_filled(rect, 0.0, Color32::WHITE.gamma_multiply(BEAT_LINE_OPACITY));
            ui.painter().text(
                rect.left_top() + egui::Vec2::new(4.0, 0.0),
                Align2::LEFT_BOTTOM,
                index,
                FontId::monospace(14.0),
                Color32::WHITE,
            );
        }

        for (index, beat_time) in ctx.secondary_beat_times().iter().enumerate() {
            if (index as u32).is_multiple_of(ctx.settings.density) {
                continue;
            }
            let rect = egui::Rect::from_center_size(
                egui::Pos2::new(
                    ctx.viewport.0.width() / 2.0 + ctx.viewport.0.min.x,
                    ctx.time_to_y(*beat_time),
                ),
                egui::Vec2::new(ctx.viewport.0.width(), 0.5),
            );
            ui.painter()
                .rect_filled(rect, 0.0, Color32::WHITE.gamma_multiply(BEAT_LINE_OPACITY));
            let numer = index as u32;
            let denom = ctx.settings.density;
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
        let viewport = world.resource::<TimelineViewport>().0.into_egui();
        let timelines = allocated_timelines(world);

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
