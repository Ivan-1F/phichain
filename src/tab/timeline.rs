use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::EguiUserTextures;
use egui::{Align2, Color32, FontId, Ui};
use num::Rational32;

use crate::assets::ImageAssets;
use crate::audio::AudioDuration;
use crate::widgets::event::event_ui;
use crate::{
    chart::{
        beat::Beat,
        event::LineEvent,
        note::{Note, NoteKind},
    },
    constants::{BASE_ZOOM, CANVAS_WIDTH, INDICATOR_POSITION},
    selection::{SelectEvent, Selected, SelectedLine},
    timing::{BpmList, ChartTime},
};

pub struct TimelineTabPlugin;

impl Plugin for TimelineTabPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimelineViewport(Rect::from_corners(Vec2::ZERO, Vec2::ZERO)))
            .insert_resource(TimelineSettings::default());
    }
}

pub fn timeline_ui_system(
    In(ui): In<&mut Ui>,
    selected_line_query: Res<SelectedLine>,
    timeline_viewport: Res<TimelineViewport>,
    bpm_list: Res<BpmList>,
    event_query: Query<(&LineEvent, &Parent, Entity, Option<&Selected>)>,
    note_query: Query<(&Note, &Parent, Entity, Option<&Selected>)>,
    mut select_events: EventWriter<SelectEvent>,
    timeline: Timeline,
    timeline_settings: Res<TimelineSettings>,
    assets: Res<ImageAssets>,
    images: Res<Assets<Image>>,
    textures: Res<EguiUserTextures>,
) {
    let selected_line = selected_line_query.0;
    let viewport = timeline_viewport;

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

    let event_timeline_viewport = viewport.event_timeline_viewport();

    for (event, parent, entity, selected) in event_query.iter() {
        if parent.get() != selected_line {
            continue;
        }

        let track = match event.kind {
            crate::chart::event::LineEventKind::X => 1,
            crate::chart::event::LineEventKind::Y => 2,
            crate::chart::event::LineEventKind::Rotation => 3,
            crate::chart::event::LineEventKind::Opacity => 4,
            crate::chart::event::LineEventKind::Speed => 5,
        };

        let x = event_timeline_viewport.width() / 5.0 * track as f32
            - event_timeline_viewport.width() / 5.0 / 2.0
            + event_timeline_viewport.min.x;
        let y = timeline.time_to_y(bpm_list.time_at(event.start_beat));

        let size = egui::Vec2::new(
            event_timeline_viewport.width() / 8000.0 * 989.0,
            timeline.duration_to_height(bpm_list.time_at(event.duration())),
        );

        let center = egui::Pos2::new(x, y - size.y / 2.0);

        if event_ui(
            ui,
            egui::Rect::from_center_size(center, size),
            event,
            selected.is_some(),
        )
        .clicked()
        {
            select_events.send(SelectEvent(entity));
        }
    }

    let note_timeline_viewport = viewport.note_timeline_viewport();

    for (note, parent, entity, selected) in note_query.iter() {
        if parent.get() != selected_line {
            continue;
        }

        let x = note_timeline_viewport.min.x
            + (note.x / CANVAS_WIDTH + 0.5) * note_timeline_viewport.width();
        let y = timeline.time_to_y(bpm_list.time_at(note.beat));

        let get_asset = |handle: &Handle<Image>| {
            (
                images.get(handle).unwrap().size(),
                textures.image_id(handle).unwrap(),
            )
        };

        let handle = match note.kind {
            NoteKind::Tap => &assets.tap,
            NoteKind::Drag => &assets.drag,
            NoteKind::Hold { .. } => &assets.hold,
            NoteKind::Flick => &assets.flick,
        };

        let (size, image) = get_asset(handle);

        let size = match note.kind {
            NoteKind::Hold { hold_beat } => egui::Vec2::new(
                note_timeline_viewport.width() / 8000.0 * size.x as f32,
                timeline.duration_to_height(bpm_list.time_at(hold_beat)),
            ),
            _ => egui::Vec2::new(
                note_timeline_viewport.width() / 8000.0 * size.x as f32,
                note_timeline_viewport.width() / 8000.0 * size.y as f32,
            ),
        };

        let center = match note.kind {
            NoteKind::Hold { hold_beat: _ } => egui::Pos2::new(x, y - size.y / 2.0),
            _ => egui::Pos2::new(x, y),
        };

        let response = ui.put(
            egui::Rect::from_center_size(center, size),
            egui::Image::new((image, size))
                .maintain_aspect_ratio(false)
                .fit_to_exact_size(size)
                .tint(if selected.is_some() {
                    Color32::LIGHT_GREEN
                } else {
                    Color32::WHITE
                })
                .sense(egui::Sense::click()),
        );

        if response.clicked() {
            select_events.send(SelectEvent(entity));
        }
    }

    for percent in timeline_settings.lane_percents() {
        ui.painter().rect_filled(
            egui::Rect::from_center_size(
                egui::Pos2::new(
                    note_timeline_viewport.min.x + note_timeline_viewport.width() * percent,
                    viewport.0.center().y,
                ),
                egui::Vec2::new(2.0, viewport.0.height()),
            ),
            0.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 40),
        );
    }

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
