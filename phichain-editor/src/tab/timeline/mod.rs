use bevy::prelude::*;
use egui::Ui;

use crate::project::project_loaded;
use crate::selection::SelectedLine;
use crate::timeline;
use crate::timeline::drag_selection::TimelineDragSelectionPlugin;
use crate::timeline::event::EventTimeline;
use crate::timeline::note::NoteTimeline;
use crate::timeline::settings::TimelineSettings;
use crate::timeline::{Timeline, TimelineItem};
use phichain_chart::note::Note;

pub struct TimelineTabPlugin;

impl Plugin for TimelineTabPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TimelineDragSelectionPlugin)
            .insert_resource(TimelineViewport(Rect::from_corners(Vec2::ZERO, Vec2::ZERO)))
            .insert_resource(TimelineSettings::default())
            .add_systems(
                Update,
                sync_timeline_line_entity_system.run_if(project_loaded()),
            );
    }
}

/// Sync timelines with [`SelectedLine`] when `timeline_settings.multi_line_editing` is off
fn sync_timeline_line_entity_system(
    selected_line: Res<SelectedLine>,
    mut timeline_settings: ResMut<TimelineSettings>,
) {
    // TODO: optimize this
    if !timeline_settings.multi_line_editing {
        // only one note timeline and one event timeline for the selected line in single line editing
        if timeline_settings.timelines.len() != 2
            || !timeline_settings
                .timelines
                .iter()
                .any(|x| matches!(x.0, TimelineItem::Note(_)))
            || !timeline_settings
                .timelines
                .iter()
                .any(|x| matches!(x.0, TimelineItem::Event(_)))
        {
            timeline_settings.timelines.clear();
            timeline_settings.timelines.push((
                TimelineItem::Note(NoteTimeline::new(selected_line.0)),
                2.0 / 3.0,
            ));
            timeline_settings
                .timelines
                .push((TimelineItem::Event(EventTimeline::new(selected_line.0)), 1.0));
        }
        // ensure all timelines are on selected line
        for (timeline, _) in &mut timeline_settings.timelines {
            match timeline {
                TimelineItem::Note(timeline) => {
                    timeline.set_line(selected_line.0);
                }
                TimelineItem::Event(timeline) => {
                    timeline.set_line(selected_line.0);
                }
            }
        }
    }
}

pub fn timeline_tab(In(ui): In<&'static mut Ui>, world: &mut World) {
    timeline::drag_selection::timeline_drag_selection(ui, world);
    let viewport = world.resource::<TimelineViewport>();
    let timeline_settings = world.resource::<TimelineSettings>();
    let rect = egui::Rect::from_min_max(
        egui::Pos2::new(viewport.0.min.x, viewport.0.min.y),
        egui::Pos2::new(viewport.0.max.x, viewport.0.max.y),
    );

    let mut viewport = rect;

    let timelines = timeline_settings.timelines.clone();

    for (timeline, percent) in &timelines {
        viewport = viewport.with_max_x(rect.min.x + percent * rect.width());
        timeline.ui(ui, world, viewport);
        viewport = viewport.with_min_x(viewport.max.x);
    }
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
