mod event_timeline;
mod note_timeline;

use bevy::prelude::*;
use egui::Ui;

use crate::constants::CANVAS_WIDTH;
use crate::selection::SelectedLine;
use crate::timeline;
use crate::timeline::event::EventTimeline;
use crate::timeline::note::NoteTimeline;
use crate::timeline::Timeline;
use phichain_chart::beat;
use phichain_chart::beat::Beat;
use phichain_chart::note::Note;

pub struct TimelineTabPlugin;

impl Plugin for TimelineTabPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugins(NoteTimelinePlugin)
            // .add_plugins(EventTimelinePlugin)
            .insert_resource(TimelineViewport(Rect::from_corners(Vec2::ZERO, Vec2::ZERO)))
            .insert_resource(TimelineSettings::default());
    }
}

pub fn timeline_tab(In(ui): In<&'static mut Ui>, world: &mut World) {
    world.resource_scope(|world: &mut World, selected_line: Mut<SelectedLine>| {
        let viewport = world.resource::<TimelineViewport>();
        let rect = egui::Rect::from_min_max(
            egui::Pos2::new(viewport.0.min.x, viewport.0.min.y),
            egui::Pos2::new(viewport.0.max.x, viewport.0.max.y),
        );
        let note_viewport = rect.with_max_x(rect.min.x + rect.width() / 3.0 * 2.0);
        let event_viewport = rect.with_min_x(rect.min.x + rect.width() / 3.0 * 2.0);
        NoteTimeline::new(selected_line.0).ui(ui, world, note_viewport);
        EventTimeline::new(selected_line.0).ui(ui, world, event_viewport);
    });
    timeline::common::beat_line_ui(ui, world);
    timeline::common::indicator_ui(ui, world);
    timeline::common::separator_ui(ui, world);
}

#[derive(Resource, Debug)]
pub struct TimelineViewport(pub Rect);

impl TimelineViewport {
    pub fn note_timeline_viewport(&self) -> Rect {
        Rect::from_corners(
            self.0.min,
            Vec2 {
                x: self.0.min.x + self.0.width() / 3.0 * 2.0,
                y: self.0.max.y,
            },
        )
    }

    pub fn event_timeline_viewport(&self) -> Rect {
        Rect::from_corners(
            Vec2 {
                x: self.0.min.x + self.0.width() / 3.0 * 2.0,
                y: self.0.min.y,
            },
            self.0.max,
        )
    }
}

pub trait TimelineFilter<T> {
    fn filter(&self, value: T) -> bool;
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NoteSideFilter {
    #[default]
    All,
    Above,
    Below,
}

impl TimelineFilter<Note> for NoteSideFilter {
    fn filter(&self, note: Note) -> bool {
        match self {
            NoteSideFilter::All => true,
            NoteSideFilter::Above => note.above,
            NoteSideFilter::Below => !note.above,
        }
    }
}

#[derive(Resource)]
pub struct TimelineSettings {
    pub zoom: f32,
    pub density: u32,
    pub lanes: u32,

    pub note_side_filter: NoteSideFilter,
}

impl Default for TimelineSettings {
    fn default() -> Self {
        Self {
            zoom: 2.0,
            density: 4,
            lanes: 11,

            note_side_filter: NoteSideFilter::default(),
        }
    }
}

impl TimelineSettings {
    pub fn attach(&self, beat: f32) -> Beat {
        beat::utils::attach(beat, self.density)
    }

    pub fn minimum_beat(&self) -> Beat {
        beat!(0, 1, self.density)
    }

    pub fn lane_percents(&self) -> Vec<f32> {
        let lane_width = 1.0 / (self.lanes + 1) as f32;
        std::iter::repeat(0)
            .take(self.lanes as usize)
            .enumerate()
            .map(|(i, _)| (i + 1) as f32 * lane_width)
            .collect()
    }

    pub fn minimum_lane(&self) -> f32 {
        CANVAS_WIDTH / self.lanes as f32
    }

    pub fn attach_x(&self, x: f32) -> f32 {
        self.lane_percents()
            .iter()
            .map(|x| (x - 0.5) * CANVAS_WIDTH)
            .min_by(|a, b| (a - x).abs().partial_cmp(&(b - x).abs()).unwrap())
            .unwrap_or(x)
    }
}
