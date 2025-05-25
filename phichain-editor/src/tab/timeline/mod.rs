use bevy::prelude::*;
use egui::{epaint, Color32, Id, Sense, Ui};

use crate::timeline;
use crate::timeline::settings::TimelineSettings;
use crate::timeline::Timeline;
use crate::utils::convert::BevyEguiConvert;
use phichain_chart::note::Note;

pub fn timeline_tab(In(mut ui): In<Ui>, world: &mut World) {
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

    // TODO bevy-0.16: make timeline_tab return Result and handle error using ? operator when register_tab supports systems returning Result
    let _ = timeline::drag_selection::timeline_drag_selection(&mut ui, world);
    let viewport = world.resource::<TimelineViewport>();
    let timeline_settings = world.resource::<TimelineSettings>();

    let timelines = timeline_settings.container.clone();

    let mut swap = None;

    for (index, item) in timelines
        .allocate(viewport.0.into_egui())
        .iter()
        .enumerate()
    {
        item.timeline.ui(&mut ui, world, item.viewport);

        let name = item.timeline.name(world);
        let galley =
            ui.painter()
                .layout_no_wrap(name.clone(), Default::default(), Default::default());

        let badge_rect = egui::Rect::from_center_size(
            egui::Pos2::new(item.viewport.center().x, 150.0),
            galley.size(),
        );

        let visuals = ui.style().visuals.widgets.inactive;

        ui.put(badge_rect, |ui: &mut Ui| {
            ui.dnd_drag_source(Id::new(&name), index, |ui| {
                ui.painter().rect(
                    badge_rect,
                    visuals.corner_radius,
                    visuals.weak_bg_fill,
                    visuals.bg_stroke,
                    epaint::StrokeKind::Inside,
                );
                ui.add(
                    egui::Label::new(egui::RichText::new(&name).color(Color32::WHITE))
                        .truncate()
                        .selectable(false),
                )
            })
            .response
        });

        // dropzone
        let response = ui.allocate_rect(item.viewport, Sense::hover());
        if let Some(dragged_payload) = response.dnd_release_payload::<usize>() {
            swap = Some((*dragged_payload, index));
        }

        if let Some(hovered_payload) = response.dnd_hover_payload::<usize>() {
            if *hovered_payload != index {
                ui.painter().rect_filled(
                    item.viewport,
                    0.0,
                    egui_dock::Style::default().overlay.selection_color,
                );
            }
        }
    }

    if let Some((from, to)) = swap {
        let mut timeline_settings = world.resource_mut::<TimelineSettings>();
        timeline_settings.container.swap(from, to);
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
