use bevy::prelude::*;
use egui::{Color32, Ui};
use url::Url;

use crate::{
    chart::{
        event::LineEvent,
        note::{Note, NoteKind},
    }, constants::CANVAS_WIDTH, misc::WorkingDirectory, selection::{SelectNoteEvent, Selected, SelectedLine}, timing::{BpmList, ChartTime}
};

pub struct TimelineTabPlugin;

impl Plugin for TimelineTabPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimelineViewport(Rect::from_corners(Vec2::ZERO, Vec2::ZERO)));
    }
}

pub fn timeline_ui_system(
    In(ui): In<&mut Ui>,
    selected_line_query: Res<SelectedLine>,
    timeline_viewport: Res<TimelineViewport>,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
    event_query: Query<&LineEvent>,
    note_query: Query<(&Note, &Parent, Entity, Option<&Selected>)>,
    working_dir: Res<WorkingDirectory>,
    mut select_events: EventWriter<SelectNoteEvent>,
) {
    let selected_line = selected_line_query.0;
    let viewport = timeline_viewport;
    let time = time.0;

    ui.painter().rect_filled(
        egui::Rect::from_center_size(
            egui::Pos2::new(
                viewport.0.width() / 2.0 + viewport.0.min.x,
                viewport.0.height() * 0.9,
            ),
            egui::Vec2::new(viewport.0.width(), 2.0),
        ),
        0.0,
        Color32::WHITE,
    );

    let event_timeline_viewport = viewport.event_timeline_viewport();

    for event in event_query.iter() {
        if event.line_id != selected_line {
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
        let y: f32 = (time - bpm_list.time_at(event.start_beat)) * 400.0 * 2.0
            + event_timeline_viewport.height() * 0.9;

        let size = egui::Vec2::new(
            event_timeline_viewport.width() / 8000.0 * 989.0,
            bpm_list.time_at(event.duration()) * 400.0 * 2.0,
        );

        let center = egui::Pos2::new(x, y - size.y / 2.0);

        ui.painter().rect(
            egui::Rect::from_center_size(center, size),
            0.0,
            Color32::LIGHT_BLUE,
            egui::Stroke::new(2.0, Color32::WHITE),
        );
    }

    let note_timeline_viewport = viewport.note_timeline_viewport();

    for (note, parent, entity, selected) in note_query.iter() {
        if parent.get() != selected_line {
            continue;
        }

        let x = (note.x / CANVAS_WIDTH + 0.5) * note_timeline_viewport.width();
        let y: f32 = (time - bpm_list.time_at(note.beat)) * 400.0 * 2.0
            + note_timeline_viewport.height() * 0.9;

        let image = match note.kind {
            NoteKind::Tap => "tap.png",
            NoteKind::Drag => "drag.png",
            NoteKind::Hold { hold_beat: _ } => {
                "hold.png"
            }
            NoteKind::Flick => "flick.png",
        };

        let image_size = match note.kind {
            NoteKind::Tap => egui::Vec2::new(989.0, 100.0),
            NoteKind::Drag => egui::Vec2::new(989.0, 60.0),
            NoteKind::Hold { hold_beat: _ } => egui::Vec2::new(989.0, 1900.0),
            NoteKind::Flick => egui::Vec2::new(989.0, 200.0),
        };

        let size = match note.kind {
            NoteKind::Hold { hold_beat } => egui::Vec2::new(
                note_timeline_viewport.width() / 8000.0 * image_size.x,
                bpm_list.time_at(hold_beat) * 400.0 * 2.0,
            ),
            _ => egui::Vec2::new(
                note_timeline_viewport.width() / 8000.0 * image_size.x,
                note_timeline_viewport.width() / 8000.0 * image_size.y,
            ),
        };

        let center = match note.kind {
            NoteKind::Hold { hold_beat: _ } => egui::Pos2::new(x, y - size.y / 2.0),
            _ => egui::Pos2::new(x, y),
        };

        let assets_dir = working_dir.0.join("assets");

        let response = ui.put(
            egui::Rect::from_center_size(center, size),
            egui::Image::new(Url::from_file_path(assets_dir.join(image)).unwrap().as_str())
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
            select_events.send(SelectNoteEvent(entity));
        }
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
