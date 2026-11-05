use bevy::prelude::*;
use bevy_persistent::Persistent;
use egui::Ui;

use crate::settings::EditorSettings;
use crate::timeline::container::{allocated_timelines, layout_timelines};
use crate::timeline::event::EventTimeline;
use crate::timeline::note::{note_timeline_ui, NoteTimeline};
use crate::timeline::settings::TimelineSettings;
use crate::timing::{Pause, Paused, Seek};
use crate::{spectrogram, timeline};
use phichain_chart::note::Note;

pub fn timeline_tab(In(mut ui): In<Ui>, world: &mut World) {
    let clip_rect = ui.clip_rect();
    let mut timeline_viewport = world.resource_mut::<TimelineViewport>();
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
    layout_timelines(world);

    // draw spectrogram background after viewport is updated
    spectrogram::draw(ui.painter(), world);

    let is_hovering = ui.rect_contains_pointer(clip_rect);
    if is_hovering {
        let (scroll_delta, is_command_pressed) = ui.ctx().input_mut(|input| {
            // using `raw_scroll_delta` since we handle smoothing ourselves
            let delta = input.raw_scroll_delta.y;
            // clear scroll delta to prevent other components from processing it
            input.raw_scroll_delta.y = 0.0;

            // `command` is Ctrl on Windows/Linux, Command on macOS
            let command_pressed = input.modifiers.command;
            (delta, command_pressed)
        });

        if scroll_delta != 0.0 {
            if is_command_pressed {
                // Ctrl/Command + scroll: adjust timeline zoom
                let mut timeline_settings = world.resource_mut::<TimelineSettings>();
                let zoom_factor = if scroll_delta > 0.0 { 1.02 } else { 1.0 / 1.02 };
                timeline_settings.zoom = (timeline_settings.zoom * zoom_factor).clamp(0.1, 5.0);
            } else {
                // normal scroll: seek
                let settings = world.resource::<Persistent<EditorSettings>>();
                world.write_message(Seek(
                    scroll_delta / 5000.0 * settings.general.timeline_scroll_sensitivity,
                ));

                let settings = world.resource::<Persistent<EditorSettings>>();
                if settings.general.pause_when_scroll && !world.resource::<Paused>().0 {
                    world.trigger(Pause);
                }
            }
        }
    }

    // TODO bevy-0.16: make timeline_tab return Result and handle error using ? operator when register_tab supports systems returning Result
    let _ = timeline::drag_selection::timeline_drag_selection(&mut ui, world);
    for (index, item) in allocated_timelines(world).into_iter().enumerate() {
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(item.viewport)
                .id_salt(item.timeline),
        );
        child.set_clip_rect(item.viewport.intersect(ui.clip_rect()));
        if world.entity(item.timeline).contains::<NoteTimeline>() {
            if let Err(error) =
                world.run_system_cached_with(note_timeline_ui, (child, item.timeline))
            {
                error!("Failed to draw note timeline: {error}");
            }
        } else if let Some(line) = item.target {
            if world.entity(item.timeline).contains::<EventTimeline>() {
                EventTimeline.ui(&mut child, world, line, item.viewport);
            }
        }
        timeline::common::timeline_badge_ui(&mut ui, world, item.timeline, item.viewport, index);
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
