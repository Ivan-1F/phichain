mod event_timeline;
mod note_timeline;

use bevy::prelude::*;
use egui::Ui;

use crate::selection::SelectedLine;
use crate::timeline;
use crate::timeline::drag_selection::TimelineDragSelectionPlugin;
use crate::timeline::event::EventTimeline;
use crate::timeline::note::NoteTimeline;
use crate::timeline::settings::TimelineSettings;
use crate::timeline::Timeline;
use phichain_chart::note::Note;

pub struct TimelineTabPlugin;

impl Plugin for TimelineTabPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugins(NoteTimelinePlugin)
            // .add_plugins(EventTimelinePlugin)
            .add_plugins(TimelineDragSelectionPlugin)
            .insert_resource(TimelineViewport(Rect::from_corners(Vec2::ZERO, Vec2::ZERO)))
            .insert_resource(TimelineSettings::default());
    }
}

pub fn timeline_tab(In(ui): In<&'static mut Ui>, world: &mut World) {
    world.resource_scope(|world: &mut World, selected_line: Mut<SelectedLine>| {
        let viewport = world.resource::<TimelineViewport>();
        let timeline_settings = world.resource::<TimelineSettings>();
        let rect = egui::Rect::from_min_max(
            egui::Pos2::new(viewport.0.min.x, viewport.0.min.y),
            egui::Pos2::new(viewport.0.max.x, viewport.0.max.y),
        );

        match (
            timeline_settings.show_note_timeline,
            timeline_settings.show_event_timeline,
        ) {
            (true, true) => {
                let note_viewport = rect.with_max_x(rect.min.x + rect.width() / 3.0 * 2.0);
                let event_viewport = rect.with_min_x(rect.min.x + rect.width() / 3.0 * 2.0);
                NoteTimeline::new(selected_line.0).ui(ui, world, note_viewport);
                EventTimeline::new(selected_line.0).ui(ui, world, event_viewport);
                timeline::common::separator_ui(ui, world);
            }
            (true, false) => {
                NoteTimeline::new(selected_line.0).ui(ui, world, rect);
            }
            (false, true) => {
                EventTimeline::new(selected_line.0).ui(ui, world, rect);
            }
            (false, false) => {}
        }
    });
    timeline::common::beat_line_ui(ui, world);
    timeline::common::indicator_ui(ui, world);
    timeline::drag_selection::timeline_drag_selection(ui, world);
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
