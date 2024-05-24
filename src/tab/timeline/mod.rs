mod event_timeline;
mod note_timeline;

use bevy::{ecs::system::SystemParam, prelude::*};
use egui::{Align2, Color32, FontId, Ui};
use num::Rational32;

use crate::audio::AudioDuration;
use crate::chart::beat;
use crate::tab::timeline::event_timeline::{
    event_timeline_drag_select_system, event_timeline_system, EventTimelinePlugin,
};
use crate::tab::timeline::note_timeline::{
    note_timeline_drag_select_system, note_timeline_system, NoteTimelinePlugin,
};
use crate::{
    chart::beat::Beat,
    constants::{BASE_ZOOM, INDICATOR_POSITION},
    timing::{BpmList, ChartTime},
};

pub struct TimelineTabPlugin;

impl Plugin for TimelineTabPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NoteTimelinePlugin)
            .add_plugins(EventTimelinePlugin)
            .insert_resource(TimelineViewport(Rect::from_corners(Vec2::ZERO, Vec2::ZERO)))
            .insert_resource(TimelineSettings::default());
    }
}

fn beat_line_ui_system(
    In(ui): In<&mut Ui>,
    viewport: Res<TimelineViewport>,
    timeline: Timeline,
    timeline_settings: Res<TimelineSettings>,
) {
    for (index, beat_time) in timeline.primary_beat_times().iter().enumerate() {
        let rect = egui::Rect::from_center_size(
            egui::Pos2::new(
                viewport.0.width() / 2.0 + viewport.0.min.x,
                timeline.time_to_y(*beat_time),
            ),
            egui::Vec2::new(viewport.0.width(), 2.0),
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

    for (index, beat_time) in timeline.secondary_beat_times().iter().enumerate() {
        if index as u32 % timeline_settings.density == 0 {
            continue;
        }
        let rect = egui::Rect::from_center_size(
            egui::Pos2::new(
                viewport.0.width() / 2.0 + viewport.0.min.x,
                timeline.time_to_y(*beat_time),
            ),
            egui::Vec2::new(viewport.0.width(), 0.5),
        );
        ui.painter().rect_filled(
            rect,
            0.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 40),
        );
        let numer = index as u32;
        let denom = timeline_settings.density;
        ui.painter().text(
            rect.left_top() + egui::Vec2::new(4.0, 0.0),
            Align2::LEFT_BOTTOM,
            format!("{}:{}/{}", numer / denom, numer % denom, denom),
            FontId::monospace(8.0),
            Color32::WHITE,
        );
    }
}

fn separator_system(In(ui): In<&mut Ui>, viewport: Res<TimelineViewport>) {
    ui.painter().rect_filled(
        egui::Rect::from_center_size(
            egui::Pos2::new(
                viewport.note_timeline_viewport().max.x,
                viewport.0.center().y,
            ),
            egui::Vec2::new(2.0, viewport.0.height()),
        ),
        0.0,
        Color32::WHITE,
    );
}

fn indicator_system(In(ui): In<&mut Ui>, viewport: Res<TimelineViewport>) {
    ui.painter().rect_filled(
        egui::Rect::from_center_size(
            egui::Pos2::new(
                viewport.0.center().x,
                viewport.0.min.y + viewport.0.height() * INDICATOR_POSITION,
            ),
            egui::Vec2::new(viewport.0.width(), 2.0),
        ),
        0.0,
        Color32::WHITE,
    );
}

pub fn timeline_tab(In(ui): In<&'static mut Ui>, world: &mut World) {
    unsafe {
        let mut system = IntoSystem::into_system(beat_line_ui_system);
        system.initialize(world);
        system.run(&mut *(ui as *mut Ui), world);

        let mut system = IntoSystem::into_system(indicator_system);
        system.initialize(world);
        system.run(&mut *(ui as *mut Ui), world);

        let mut system = IntoSystem::into_system(separator_system);
        system.initialize(world);
        system.run(&mut *(ui as *mut Ui), world);

        let mut system = IntoSystem::into_system(note_timeline_drag_select_system);
        system.initialize(world);
        system.run(&mut *(ui as *mut Ui), world);

        let mut system = IntoSystem::into_system(note_timeline_system);
        system.initialize(world);
        system.run(&mut *(ui as *mut Ui), world);

        let mut system = IntoSystem::into_system(event_timeline_drag_select_system);
        system.initialize(world);
        system.run(&mut *(ui as *mut Ui), world);

        let mut system = IntoSystem::into_system(event_timeline_system);
        system.initialize(world);
        system.run(&mut *(ui as *mut Ui), world);
    }
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

#[derive(Resource)]
pub struct TimelineSettings {
    pub zoom: f32,
    pub density: u32,
    pub lanes: u32,
}

impl Default for TimelineSettings {
    fn default() -> Self {
        Self {
            zoom: 2.0,
            density: 4,
            lanes: 11,
        }
    }
}

impl TimelineSettings {
    pub fn attach(&self, beat: f32) -> Beat {
        beat::utils::attach(beat, self.density)
    }

    pub fn minimum_beat(&self) -> Beat {
        Beat::new(0, Rational32::new(1, self.density as i32))
    }

    pub fn lane_percents(&self) -> Vec<f32> {
        let lane_width = 1.0 / (self.lanes + 1) as f32;
        std::iter::repeat(0)
            .take(self.lanes as usize)
            .enumerate()
            .map(|(i, _)| (i + 1) as f32 * lane_width)
            .collect()
    }
}

#[derive(SystemParam)]
pub struct Timeline<'w> {
    bpm_list: Res<'w, BpmList>,
    timeline_settings: Res<'w, TimelineSettings>,
    current_time: Res<'w, ChartTime>,
    viewport: Res<'w, TimelineViewport>,

    audio_duration: Res<'w, AudioDuration>,
}

impl<'w> Timeline<'w> {
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
                self.bpm_list.time_at(Beat::new(
                    0,
                    Rational32::new(i as i32, self.timeline_settings.density as i32),
                ))
            })
            .take_while(|x| x <= &self.audio_duration.0.as_secs_f32())
            .collect()
    }

    pub fn time_to_y(&self, time: f32) -> f32 {
        (self.current_time.0 - time) * BASE_ZOOM * self.timeline_settings.zoom
            + self.viewport.0.min.y
            + self.viewport.0.height() * INDICATOR_POSITION
    }

    pub fn y_to_time(&self, y: f32) -> f32 {
        self.current_time.0
            - (y - (self.viewport.0.min.y + self.viewport.0.height() * INDICATOR_POSITION))
                / (BASE_ZOOM * self.timeline_settings.zoom)
    }

    pub fn duration_to_height(&self, duration: f32) -> f32 {
        duration * BASE_ZOOM * self.timeline_settings.zoom
    }
}
