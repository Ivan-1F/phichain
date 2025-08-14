use bevy::prelude::*;
use bevy_persistent::Persistent;
use egui::Ui;

use crate::spectrogram::draw;
use crate::settings::EditorSettings;
use crate::timeline;
use crate::timeline::settings::TimelineSettings;
use crate::timeline::Timeline;
use crate::timing::{PauseEvent, Paused, SeekEvent};
use crate::utils::convert::BevyEguiConvert;
use phichain_chart::note::Note;

pub fn timeline_tab(In(mut ui): In<Ui>, world: &mut World) {
    draw(ui.painter(), world);

    let mut timeline_viewport = world.resource_mut::<TimelineViewport>();
    let clip_rect = ui.clip_rect();
    timeline_viewport.0 = Rect::from_corners(
        Vec2 {
            x: clip_rect.min.x,
            y: clip_rect.min.y,
        },
        Vec2 {
            x: clip_rect.max.x,
            y: clip_rect.max.y,
        },
    );

    let is_hovering = ui.rect_contains_pointer(clip_rect);
    if is_hovering {
        let scroll_delta = ui.ctx().input_mut(|input| {
            // using `raw_scroll_delta` since we handle smoothing ourselves
            let delta = input.raw_scroll_delta.y;
            // clear scroll delta to prevent other components from processing it
            input.raw_scroll_delta.y = 0.0;

            delta
        });

        if scroll_delta != 0.0 {
            let settings = world.resource::<Persistent<EditorSettings>>();
            world.send_event(SeekEvent(
                scroll_delta / 5000.0 * settings.general.timeline_scroll_sensitivity,
            ));

            let settings = world.resource::<Persistent<EditorSettings>>();
            if settings.general.pause_when_scroll && !world.resource::<Paused>().0 {
                world.send_event_default::<PauseEvent>();
            }
        }
    }

    // TODO bevy-0.16: make timeline_tab return Result and handle error using ? operator when register_tab supports systems returning Result
    let _ = timeline::drag_selection::timeline_drag_selection(&mut ui, world);
    let viewport = world.resource::<TimelineViewport>();
    let timeline_settings = world.resource::<TimelineSettings>();

    let timelines = timeline_settings.container.clone();

    for (index, item) in timelines
        .allocate(viewport.0.into_egui())
        .iter()
        .enumerate()
    {
        item.timeline.ui(&mut ui, world, item.viewport);
        timeline::common::timeline_badge_ui(&mut ui, world, item, index);
    }

    timeline::common::beat_line_ui(&mut ui, world);
    timeline::common::indicator_ui(&mut ui, world);
    timeline::common::separator_ui(&mut ui, world);
    timeline::common::bpm_change_ui(&mut ui, world);
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
