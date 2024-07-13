use bevy::prelude::*;
use egui::Ui;

use crate::timeline;
use crate::timeline::settings::TimelineSettings;
use crate::timeline::Timeline;
use crate::utils::convert::BevyEguiConvert;
use phichain_chart::note::Note;

pub fn timeline_tab(In(ui): In<&mut Ui>, world: &mut World) {
    timeline::drag_selection::timeline_drag_selection(ui, world);
    let viewport = world.resource::<TimelineViewport>();
    let timeline_settings = world.resource::<TimelineSettings>();

    let timelines = timeline_settings.container.clone();

    for item in &timelines.allocate(viewport.0.into_egui()) {
        item.timeline.ui(ui, world, item.viewport);
    }
    timeline::common::beat_line_ui(ui, world);
    timeline::common::indicator_ui(ui, world);
    timeline::common::separator_ui(ui, world);
}

#[derive(Resource, Debug)]
pub struct TimelineViewport(pub Rect);

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
