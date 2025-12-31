use crate::constants::INDICATOR_POSITION;
use crate::editing::command::event::EditEvent;
use crate::editing::command::EditorCommand;
use crate::editing::pending::Pending;
use crate::editing::DoCommandEvent;
use crate::selection::{SelectEvent, Selected, SelectedLine};
use crate::timeline::{Timeline, TimelineContext};
use crate::timing::SeekToEvent;
use crate::ui::widgets::beat_range_drag_zone::BeatRangeDragZone;
use bevy::ecs::system::SystemState;
use bevy::prelude::{Entity, EventWriter, Query, Res, World};
use egui::{Align2, Color32, FontId, Rect, Sense, Stroke, StrokeKind, Ui};
use phichain_chart::bpm_list::BpmList;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
use phichain_chart::line::Line;
use phichain_game::event::{EventOf, Events};
use std::iter;

#[derive(Debug, Clone)]
pub struct EventTimeline(pub Option<Entity>);

impl EventTimeline {
    pub fn new(line: Entity) -> Self {
        Self(Some(line))
    }

    pub fn new_binding() -> Self {
        Self(None)
    }

    pub fn line_entity(&self, world: &mut World) -> Entity {
        self.line_entity_from_fallback(world.resource::<SelectedLine>().0)
    }

    pub fn line_entity_from_fallback(&self, fallback: Entity) -> Entity {
        self.0.unwrap_or(fallback)
    }
}

#[derive(Debug, Clone)]
struct EventTrackData<T> {
    x: T,
    y: T,
    rotation: T,
    opacity: T,
    speed: T,
}

impl<T> EventTrackData<T> {
    fn get(&self, kind: LineEventKind) -> &T {
        match kind {
            LineEventKind::X => &self.x,
            LineEventKind::Y => &self.y,
            LineEventKind::Rotation => &self.rotation,
            LineEventKind::Opacity => &self.opacity,
            LineEventKind::Speed => &self.speed,
        }
    }

    fn get_mut(&mut self, kind: LineEventKind) -> &mut T {
        match kind {
            LineEventKind::X => &mut self.x,
            LineEventKind::Y => &mut self.y,
            LineEventKind::Rotation => &mut self.rotation,
            LineEventKind::Opacity => &mut self.opacity,
            LineEventKind::Speed => &mut self.speed,
        }
    }

    fn iter(&self) -> impl Iterator<Item = (LineEventKind, &T)> {
        [
            (LineEventKind::X, &self.x),
            (LineEventKind::Y, &self.y),
            (LineEventKind::Rotation, &self.rotation),
            (LineEventKind::Opacity, &self.opacity),
            (LineEventKind::Speed, &self.speed),
        ]
        .into_iter()
    }
}

impl<T: Clone> EventTrackData<T> {
    fn splat(value: T) -> Self {
        Self {
            x: value.clone(),
            y: value.clone(),
            rotation: value.clone(),
            opacity: value.clone(),
            speed: value.clone(),
        }
    }
}

impl<T: PartialEq> EventTrackData<T> {
    fn contains(&self, value: &T) -> bool {
        &self.x == value
            || &self.y == value
            || &self.rotation == value
            || &self.opacity == value
            || &self.speed == value
    }
}

impl Timeline for EventTimeline {
    fn ui(&self, ui: &mut Ui, world: &mut World, viewport: Rect) {
        // lane
        // [0.2, 0.4, 0.6, 0.8]
        let lane_percents = iter::repeat_n(0.0, 5 - 1)
            .enumerate()
            .map(|(i, _)| (i + 1) as f32 * 1.0 / 5.0)
            .collect::<Vec<_>>();
        for percent in lane_percents {
            ui.painter().rect_filled(
                Rect::from_center_size(
                    egui::Pos2::new(
                        viewport.min.x + viewport.width() * percent,
                        viewport.center().y,
                    ),
                    egui::Vec2::new(2.0, viewport.height()),
                ),
                0.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, 40),
            );
        }

        let line_entity = self.line_entity(world);

        let mut state: SystemState<(
            TimelineContext,
            Query<(&mut LineEvent, Entity, Option<&Selected>, Option<&Pending>)>,
            Query<&Events>,
            Res<BpmList>,
            EventWriter<SelectEvent>,
            EventWriter<DoCommandEvent>,
            EventWriter<SeekToEvent>,
        )> = SystemState::new(world);

        let (
            ctx,
            mut event_query,
            events_query,
            bpm_list,
            mut select_events,
            mut event_writer,
            mut seek_to,
        ) = state.get_mut(world);

        let compute_x = |track: u8| -> f32 {
            viewport.width() / 5.0 * track as f32 - viewport.width() / 5.0 / 2.0 + viewport.min.x
        };

        let get_event_rect = |event: &LineEvent| {
            let x = compute_x(event.kind.into());
            let y = ctx.time_to_y(bpm_list.time_at(event.start_beat));

            let size = egui::Vec2::new(
                viewport.width() / 8000.0 * 989.0,
                y - ctx.time_to_y(bpm_list.time_at(event.end_beat)),
            );

            let center = egui::Pos2::new(x, y - size.y / 2.0);

            Rect::from_center_size(center, size)
        };

        // in viewport, but start value outside bottom
        let mut start_outside_bottom = EventTrackData::splat(None::<Entity>);

        // in viewport, but end value outside bottom
        let mut end_outside_bottom = EventTrackData::splat(None::<Entity>);

        // outside bottom
        let mut first_events_outside_bottom = EventTrackData::splat(None::<Entity>);
        let mut first_events_outside_bottom_y = EventTrackData::splat(f32::MAX);

        // outside top
        let mut first_events_outside_top = EventTrackData::splat(None::<Entity>);
        let mut first_events_outside_top_y = EventTrackData::splat(f32::MIN);

        let Ok(events) = events_query.get(line_entity) else {
            return;
        };

        let viewport_margin = 100.0;

        for entity in events.iter() {
            let (mut event, entity, selected, pending) = event_query.get_mut(*entity).unwrap();

            let rect = get_event_rect(&event);

            if rect.bottom() >= viewport.bottom() && rect.top() <= viewport.bottom() {
                // in viewport, but start value outside bottom
                *start_outside_bottom.get_mut(event.kind) = Some(entity);
            }

            if rect.top() <= viewport.top() && rect.bottom() >= viewport.top() {
                // in viewport, but start value outside bottom
                *end_outside_bottom.get_mut(event.kind) = Some(entity);
            }

            if rect.top() >= viewport.bottom() {
                // outside bottom
                if rect.top() < *first_events_outside_bottom_y.get(event.kind) {
                    *first_events_outside_bottom_y.get_mut(event.kind) = rect.top();
                    *first_events_outside_bottom.get_mut(event.kind) = Some(entity);
                }
            }

            if rect.bottom() <= viewport.top() {
                // outside top
                if rect.bottom() > *first_events_outside_top_y.get(event.kind) {
                    *first_events_outside_top_y.get_mut(event.kind) = rect.bottom();
                    *first_events_outside_top.get_mut(event.kind) = Some(entity);
                }
            }

            // skip rendering work for events far outside of the viewport unless they are
            // referenced by one of the boundary helpers (next/previous event indicators).
            let is_far_outside = rect.bottom() < viewport.top() - viewport_margin
                || rect.top() > viewport.bottom() + viewport_margin;
            if is_far_outside
                && !start_outside_bottom.contains(&Some(entity))
                && !end_outside_bottom.contains(&Some(entity))
                && !first_events_outside_bottom.contains(&Some(entity))
                && !first_events_outside_top.contains(&Some(entity))
            {
                continue;
            }

            let mut color = if selected.is_some() {
                Color32::LIGHT_GREEN
            } else {
                match event.value {
                    LineEventValue::Transition { .. } => Color32::LIGHT_BLUE,
                    LineEventValue::Constant(_) => Color32::LIGHT_RED,
                }
            };

            if pending.is_some() {
                color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 20);
            }

            let mut on_event_change = |old_event, new_event| {
                event_writer.write(DoCommandEvent(EditorCommand::EditEvent(EditEvent::new(
                    entity, old_event, new_event,
                ))));
            };

            let response = ui.allocate_rect(rect, Sense::click());
            if ui.is_rect_visible(rect)
                || first_events_outside_bottom.contains(&Some(entity))
                || first_events_outside_top.contains(&Some(entity))
            {
                ui.painter().rect(
                    rect,
                    0.0,
                    color,
                    Stroke::new(2.0, color.gamma_multiply(1.2)),
                    StrokeKind::Middle,
                );

                if let Some(drag) = BeatRangeDragZone::new(
                    rect,
                    "event-drag",
                    &ctx,
                    &mut *event,
                    |e| e.start_beat.value(),
                    |e| e.end_beat.value(),
                    |e, b| e.start_beat = b,
                    |e, b| e.end_beat = b,
                )
                .show(ui)
                {
                    on_event_change(drag.from, drag.to);
                }

                ui.painter().text(
                    if end_outside_bottom.contains(&Some(entity)) {
                        rect.center_top()
                            .max(egui::Pos2::new(rect.center_top().x, viewport.top()))
                    } else {
                        rect.center_top()
                            .min(egui::Pos2::new(rect.center_top().x, viewport.max.y - 18.0))
                    },
                    Align2::CENTER_TOP,
                    event.value.end(),
                    FontId::default(),
                    Color32::DARK_GREEN,
                );
                ui.painter().text(
                    if start_outside_bottom.contains(&Some(entity)) {
                        rect.center_bottom()
                            .min(egui::Pos2::new(rect.center_bottom().x, viewport.max.y))
                    } else {
                        rect.center_bottom().max(egui::Pos2::new(
                            rect.center_bottom().x,
                            viewport.top() + 18.0,
                        ))
                    },
                    Align2::CENTER_BOTTOM,
                    event.value.start(),
                    FontId::default(),
                    Color32::DARK_GREEN,
                );
            }

            if response.clicked() {
                select_events.write(SelectEvent(vec![entity]));
            }
        }

        let mut make_anchor = |kind: LineEventKind, event: Entity, top: bool| {
            let x = compute_x(kind.into());
            let size = egui::Vec2::new(viewport.width() / 8000.0 * 989.0, 10.0);
            let center = egui::Pos2::new(
                x,
                if top {
                    viewport.top() + 10.0 / 2.0
                } else {
                    viewport.bottom() - 10.0 / 2.0
                },
            );

            if ui
                .allocate_rect(Rect::from_center_size(center, size), Sense::click())
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .on_hover_text(t!("tab.timeline.event.jump_to_event"))
                .clicked()
            {
                if let Ok(event) = event_query.get(event) {
                    // TODO: refactor logic for navigation
                    seek_to.write(SeekToEvent(bpm_list.time_at(event.0.start_beat)));
                }
            }
        };

        for (kind, event) in first_events_outside_bottom.iter() {
            if let Some(event) = event {
                make_anchor(kind, *event, false);
            }
        }

        for (kind, event) in first_events_outside_top.iter() {
            if let Some(event) = event {
                make_anchor(kind, *event, true);
            }
        }

        // event track type indicator banner
        ui.painter().rect(
            Rect::from_two_pos(
                egui::Pos2::new(viewport.min.x, viewport.max.y * INDICATOR_POSITION + 10.0),
                egui::Pos2::new(viewport.max.x, viewport.max.y * INDICATOR_POSITION + 40.0),
            ),
            0.0,
            Color32::BLACK,
            Stroke::NONE,
            StrokeKind::Middle,
        );

        // event track type indicator
        ui.style_mut().interaction.selectable_labels = false;
        for (i, txt) in [
            "X",
            "Y",
            egui_phosphor::regular::ARROWS_CLOCKWISE,
            egui_phosphor::regular::CIRCLE_HALF,
            egui_phosphor::regular::GAUGE,
        ]
        .iter()
        .enumerate()
        {
            ui.put(
                Rect::from_center_size(
                    egui::Pos2::new(
                        viewport.min.x
                            + viewport.width() / 5.0 * i as f32
                            + viewport.width() / 10.0,
                        viewport.max.y * INDICATOR_POSITION + 20.0,
                    ),
                    egui::Vec2::splat(10.0),
                ),
                egui::Label::new(egui::RichText::new(*txt).color(Color32::WHITE).size(20.0)),
            );
        }
        ui.style_mut().interaction.selectable_labels = true;
    }

    fn on_drag_selection(&self, world: &mut World, viewport: Rect, selection: Rect) -> Vec<Entity> {
        let line_entity = self.line_entity(world);

        let x_range = selection.x_range();
        let time_range = selection.y_range();

        let mut state: SystemState<(Query<(&LineEvent, &EventOf, Entity)>, Res<BpmList>)> =
            SystemState::new(world);
        let (event_query, bpm_list) = state.get_mut(world);

        event_query
            .iter()
            .filter(|x| x.1.target() == line_entity)
            .filter(|x| {
                let event = x.0;
                let track: u8 = event.kind.into();
                let target_x = (track - 1) as f32 * (1.0 / 5.0) + (1.0 / (5.0 * 2.0));
                x_range.contains(target_x * viewport.width())
                    && time_range.contains(bpm_list.time_at(event.start_beat))
            })
            .map(|x| x.2)
            .collect()
    }

    fn name(&self, world: &World) -> String {
        match self.0 {
            None => format!(
                "{} {}",
                egui_phosphor::regular::DIAMOND,
                t!("tab.timeline_setting.timelines.binding")
            ),
            Some(entity) => format!(
                "{} {}",
                egui_phosphor::regular::DIAMOND,
                world.get::<Line>(entity).unwrap().name
            ),
        }
    }
}
