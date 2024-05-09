use bevy::prelude::*;
use egui::{Color32, Ui};

use crate::{
    chart::{
        event::LineEvent,
        note::{Note, NoteKind},
    },
    selection::SelectedLine,
    timing::ChartTime,
};

pub struct TimelineTabPlugin;

impl Plugin for TimelineTabPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimelineViewport(Rect::from_corners(Vec2::ZERO, Vec2::ZERO)));
    }
}

pub fn event_timeline_ui(ui: &mut Ui, world: &mut World) {
    let selected_line = world.resource::<SelectedLine>().0;
    let viewport = world
        .resource::<TimelineViewport>()
        .event_timeline_viewport();
    let time = world.resource::<ChartTime>().0;

    ui.painter().rect_filled(
        egui::Rect::from_center_size(
            egui::Pos2::new(
                viewport.width() / 2.0 + viewport.min.x,
                viewport.height() * 0.9,
            ),
            egui::Vec2::new(viewport.width(), 2.0),
        ),
        0.0,
        Color32::WHITE,
    );

    for event in world.query::<&LineEvent>().iter(world) {
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

        let x =
            viewport.width() / 5.0 * track as f32 - viewport.width() / 5.0 / 2.0 + viewport.min.x;
        let y: f32 = (time - event.start_beat.value() * (60.0 / 174.0)) * 400.0 * 2.0
            + viewport.height() * 0.9;

        let size = egui::Vec2::new(
            viewport.width() / 8000.0 * 989.0,
            event.duration().value() * (60.0 / 174.0) * 400.0 * 2.0,
        );

        let center = egui::Pos2::new(x, y - size.y / 2.0);

        ui.painter().rect(egui::Rect::from_center_size(center, size), 0.0, Color32::LIGHT_BLUE, egui::Stroke::new(2.0, Color32::WHITE));
    }
}

pub fn note_timeline_ui(ui: &mut Ui, world: &mut World) {
    let selected_line = world.resource::<SelectedLine>().0;
    let viewport = world
        .resource::<TimelineViewport>()
        .note_timeline_viewport();
    let time = world.resource::<ChartTime>().0;

    ui.painter().rect_filled(
        egui::Rect::from_center_size(
            egui::Pos2::new(viewport.width() / 2.0, viewport.height() * 0.9),
            egui::Vec2::new(viewport.width(), 2.0),
        ),
        0.0,
        Color32::WHITE,
    );

    for (note, parent) in world.query::<(&Note, &Parent)>().iter(world) {
        if parent.get() != selected_line {
            continue;
        }

        let x = (note.x + 0.5) * viewport.width();
        let y: f32 =
            (time - note.beat.value() * (60.0 / 174.0)) * 400.0 * 2.0 + viewport.height() * 0.9;

        let image = match note.kind {
            NoteKind::Tap => egui::include_image!("../../assets/tap.png"),
            NoteKind::Drag => egui::include_image!("../../assets/drag.png"),
            NoteKind::Hold { hold_beat: _ } => {
                egui::include_image!("../../assets/hold.png")
            }
            NoteKind::Flick => egui::include_image!("../../assets/flick.png"),
        };

        let image_size = match note.kind {
            NoteKind::Tap => egui::Vec2::new(989.0, 100.0),
            NoteKind::Drag => egui::Vec2::new(989.0, 60.0),
            NoteKind::Hold { hold_beat: _ } => egui::Vec2::new(989.0, 1900.0),
            NoteKind::Flick => egui::Vec2::new(989.0, 200.0),
        };

        let size = match note.kind {
            NoteKind::Hold { hold_beat } => egui::Vec2::new(
                viewport.width() / 8000.0 * image_size.x,
                hold_beat.value() * (60.0 / 174.0) * 400.0 * 2.0,
            ),
            _ => egui::Vec2::new(
                viewport.width() / 8000.0 * image_size.x,
                viewport.width() / 8000.0 * image_size.y,
            ),
        };

        let center = match note.kind {
            NoteKind::Hold { hold_beat: _ } => egui::Pos2::new(x, y - size.y / 2.0),
            _ => egui::Pos2::new(x, y),
        };

        let response = ui.put(
            egui::Rect::from_center_size(center, size),
            egui::Image::new(image)
                .maintain_aspect_ratio(false)
                .fit_to_exact_size(size)
                .sense(egui::Sense::click()),
        );

        if response.clicked() {
            println!("{:?}", note);
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
