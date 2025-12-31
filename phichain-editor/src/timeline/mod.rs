pub mod container;
pub mod drag_selection;
pub mod event;
pub mod note;
pub mod settings;

use crate::constants::{BASE_ZOOM, INDICATOR_POSITION};
use crate::tab::timeline::TimelineViewport;
use crate::timeline::drag_selection::TimelineDragSelectionPlugin;
use crate::timeline::event::EventTimeline;
use crate::timeline::note::NoteTimeline;
use crate::timeline::settings::TimelineSettings;
use crate::timing::ChartTime;
use bevy::app::{App, Plugin};
use bevy::ecs::component::HookContext;
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::DeferredWorld;
use bevy::math::Vec2;
use bevy::prelude::*;
use egui::{Rect, Ui};
use enum_dispatch::enum_dispatch;
use phichain_chart::beat;
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::line::Line;
use phichain_game::audio::AudioDuration;

pub struct TimelinePlugin;

impl Plugin for TimelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TimelineDragSelectionPlugin)
            .insert_resource(TimelineViewport(bevy::math::Rect::from_corners(
                Vec2::ZERO,
                Vec2::ZERO,
            )))
            .insert_resource(TimelineSettings::default());

        app.world_mut()
            .register_component_hooks::<Line>()
            .on_remove(clean_dangle_timelines);
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

    pub fn beat_f32_to_y(&self, beat: f32) -> f32 {
        self.beat_to_y(Beat::from(beat))
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

    fn name(&self, world: &World) -> String;
}

#[enum_dispatch]
#[derive(Debug, Clone)]
pub enum TimelineItem {
    Note(NoteTimeline),
    Event(EventTimeline),
}

impl TimelineItem {
    pub fn line_entity(&self) -> Option<Entity> {
        match self {
            TimelineItem::Note(timeline) => timeline.0,
            TimelineItem::Event(timeline) => timeline.0,
        }
    }
}

pub mod common {
    use crate::constants::INDICATOR_POSITION;
    use crate::timeline::container::AllocatedTimeline;
    use crate::timeline::settings::TimelineSettings;
    use crate::timeline::Timeline;
    use crate::timeline::TimelineContext;
    use bevy::ecs::system::SystemState;
    use bevy::prelude::*;
    use egui::{epaint, Align2, Color32, FontId, Id, Sense, Ui};
    use phichain_chart::bpm_list::BpmList;

    pub fn timeline_badge_ui(
        ui: &mut Ui,
        world: &mut World,
        item: &AllocatedTimeline,
        index: usize,
    ) {
        let mut swap = None;

        let name = item.timeline.name(world);
        let galley =
            ui.painter()
                .layout_no_wrap(name.clone(), Default::default(), Default::default());

        let badge_rect = egui::Rect::from_center_size(
            egui::Pos2::new(
                item.viewport.center().x,
                item.viewport.top() + item.viewport.height() * 0.1,
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
        let response = ui.allocate_rect(item.viewport, Sense::hover());
        if let Some(dragged_payload) = response.dnd_release_payload::<usize>() {
            swap = Some((*dragged_payload, index));
        }

        if let Some(hovered_payload) = response.dnd_hover_payload::<usize>() {
            if *hovered_payload != index {
                ui.painter().rect_filled(
                    item.viewport,
                    0.0,
                    egui_dock::Style::default().overlay.selection_color,
                );
            }
        }

        if let Some((from, to)) = swap {
            let mut timeline_settings = world.resource_mut::<TimelineSettings>();
            timeline_settings.container.swap(from, to);
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
            ui.painter().rect_filled(
                rect,
                0.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, 40),
            );
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
        let mut state: SystemState<TimelineContext> = SystemState::new(world);
        let mut ctx = state.get_mut(world);

        let timelines = ctx.settings.container.timelines.clone();

        for (index, timeline) in timelines.iter().enumerate() {
            let rect = egui::Rect::from_center_size(
                egui::Pos2::new(
                    ctx.viewport.0.min.x + timeline.fraction * ctx.viewport.0.width(),
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
                // TODO: make ManagedTimeline a handle so we can use `timeline.offset(delta_percent)`
                ctx.settings.container.offset_timeline(index, delta_percent);
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

fn clean_dangle_timelines(mut world: DeferredWorld, context: HookContext) {
    let mut timeline_settings = world.resource_mut::<TimelineSettings>();

    if let Some(index) =
        timeline_settings
            .container
            .timelines
            .iter()
            .position(|x| match &x.timeline {
                TimelineItem::Note(timeline) => timeline.0 == Some(context.entity),
                TimelineItem::Event(timeline) => timeline.0 == Some(context.entity),
            })
    {
        info!("Removed timeline due to removal of line");
        timeline_settings.container.remove(index);
    }
}
