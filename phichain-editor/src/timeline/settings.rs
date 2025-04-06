use crate::tab::timeline::NoteSideFilter;
use crate::timeline::container::TimelineContainer;
use bevy::prelude::Resource;
use phichain_chart::beat;
use phichain_chart::beat::Beat;
use phichain_chart::constants::CANVAS_WIDTH;

#[derive(Resource)]
pub struct TimelineSettings {
    pub zoom: f32,
    pub density: u32,
    pub lanes: u32,

    pub container: TimelineContainer,

    pub note_side_filter: NoteSideFilter,
}

impl Default for TimelineSettings {
    fn default() -> Self {
        Self {
            zoom: 2.0,
            density: 4,
            lanes: 11,

            container: TimelineContainer::default(),

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
        std::iter::repeat_n(0, self.lanes as usize)
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
