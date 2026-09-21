use bevy::prelude::*;
use bevy_persistent::Persistent;
use egui::Ui;

use crate::constants::INDICATOR_POSITION;
use crate::settings::EditorSettings;
use crate::timeline::settings::TimelineSettings;
use crate::timeline::Timeline;
use crate::timing::{Pause, Paused, Seek};
use crate::utils::convert::BevyEguiConvert;
use crate::{spectrogram, timeline};
use phichain_chart::note::Note;
use phichain_game::curve_note_track::CurveNoteTrackFrom;
use phichain_game::Pending;

pub fn timeline_tab(In(mut ui): In<Ui>, world: &mut World) {
    let clip_rect = ui.clip_rect();
    if let Ok(preview) = world
        .query_filtered::<Entity, (With<Pending>, With<CurveNoteTrackFrom>)>()
        .single(world)
    {
        let viewport = world
            .resource::<TimelineSettings>()
            .container
            .allocate(clip_rect)
            .into_iter()
            .find(|item| matches!(item.timeline, timeline::TimelineItem::Note(_)))
            .map(|item| item.viewport)
            .unwrap_or(clip_rect);
        let escape = world
            .resource::<ButtonInput<KeyCode>>()
            .just_pressed(KeyCode::Escape);
        let cancel = escape
            || egui::Area::new(ui.id().with("pending_curve_note_track"))
                .order(egui::Order::Foreground)
                .pivot(egui::Align2::CENTER_CENTER)
                .fixed_pos(egui::pos2(
                    viewport.center().x,
                    viewport.top() + viewport.height() * (INDICATOR_POSITION + 1.0) / 2.0,
                ))
                .constrain_to(viewport)
                .show(ui.ctx(), |ui| {
                    ui.set_width((viewport.width() - 16.0).clamp(0.0, 280.0));
                    for font in ui.style_mut().text_styles.values_mut() {
                        font.size *= 0.9;
                    }
                    ui.spacing_mut().interact_size.y *= 0.9;
                    egui::Frame::popup(ui.style())
                        .inner_margin(egui::Margin::symmetric(10, 6))
                        .show(ui, |ui| {
                            let (_, cancel) = egui::Sides::new().shrink_left().wrap().show(
                                ui,
                                |ui| {
                                    ui.strong(t!("tab.timeline.curve_note_track.creating"));
                                },
                                |ui| {
                                    ui.add(
                                        egui::Button::new(t!(
                                            "tab.timeline.curve_note_track.cancel"
                                        ))
                                        .frame(false),
                                    )
                                    .on_hover_text("Esc")
                                },
                            );
                            ui.small(t!("tab.timeline.curve_note_track.select_destination"));
                            cancel.clicked()
                        })
                        .inner
                })
                .inner;
        if cancel {
            world.despawn(preview);
        }
    }
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
    let viewport = world.resource::<TimelineViewport>();
    let timeline_settings = world.resource::<TimelineSettings>();

    let timelines = timeline_settings.container.clone();

    for (index, item) in timelines
        .allocate(viewport.0.into_egui())
        .iter()
        .enumerate()
    {
        ui.push_id(("timeline", index), |ui| {
            item.timeline.ui(ui, world, item.viewport);
        });
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
