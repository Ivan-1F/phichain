use crate::constants::INDICATOR_POSITION;
use crate::editing::command::event::EditEvent;
use crate::editing::command::EditorCommand;
use crate::editing::pending::Pending;
use crate::editing::DoCommandEvent;
use crate::selection::{SelectEvent, Selected, SelectedLine};
use crate::timeline::{Timeline, TimelineContext};
use bevy::ecs::system::SystemState;
use bevy::hierarchy::Parent;
use bevy::prelude::{Entity, EventWriter, Query, Res, World};
use egui::{Align2, Color32, FontId, Rangef, Rect, Sense, Stroke, Ui};
use phichain_chart::bpm_list::BpmList;
use phichain_chart::event::LineEvent;
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

impl Timeline for EventTimeline {
    fn ui(&self, ui: &mut Ui, world: &mut World, viewport: Rect) {
        let line_entity = self.line_entity(world);

        let mut state: SystemState<(
            TimelineContext,
            Query<(
                &mut LineEvent,
                &Parent,
                Entity,
                Option<&Selected>,
                Option<&Pending>,
            )>,
            Res<BpmList>,
            EventWriter<SelectEvent>,
            EventWriter<DoCommandEvent>,
        )> = SystemState::new(world);

        let (ctx, mut event_query, bpm_list, mut select_events, mut event_writer) =
            state.get_mut(world);

        for (mut event, parent, entity, selected, pending) in &mut event_query {
            if parent.get() != line_entity {
                continue;
            }

            let track: u8 = event.kind.into();

            let x = viewport.width() / 5.0 * track as f32 - viewport.width() / 5.0 / 2.0
                + viewport.min.x;
            let y = ctx.time_to_y(bpm_list.time_at(event.start_beat));

            let size = egui::Vec2::new(
                viewport.width() / 8000.0 * 989.0,
                y - ctx.time_to_y(bpm_list.time_at(event.end_beat)),
            );

            let center = egui::Pos2::new(x, y - size.y / 2.0);

            let mut color = if selected.is_some() {
                Color32::LIGHT_GREEN
            } else {
                Color32::LIGHT_BLUE
            };

            if pending.is_some() {
                color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 20);
            }

            let mut on_event_change = |old_event, new_event| {
                event_writer.send(DoCommandEvent(EditorCommand::EditEvent(EditEvent::new(
                    entity, old_event, new_event,
                ))));
            };

            let rect = Rect::from_center_size(center, size);

            let response = ui.allocate_rect(rect, Sense::click());
            if ui.is_rect_visible(rect) {
                ui.painter()
                    .rect(rect, 0.0, color, Stroke::new(2.0, Color32::WHITE));

                let mut make_drag_zone = |start: bool| {
                    let drag_zone = Rect::from_x_y_ranges(
                        rect.x_range(),
                        if start {
                            Rangef::from(rect.max.y - 5.0..=rect.max.y)
                        } else {
                            Rangef::from(rect.min.y..=rect.min.y + 5.0)
                        },
                    );
                    let response = ui
                        .allocate_rect(drag_zone, Sense::drag())
                        .on_hover_and_drag_cursor(egui::CursorIcon::ResizeVertical);

                    if response.drag_started() {
                        ui.data_mut(|data| data.insert_temp(egui::Id::new("event-drag"), *event));
                    }

                    if response.dragged() {
                        let drag_delta = response.drag_delta();

                        if start {
                            let new_y = ctx.beat_to_y(event.start_beat) + drag_delta.y;
                            let new_beat = ctx.y_to_beat_f32(new_y);
                            // will be attached when stop dragging
                            *event.start_beat.float_mut() += new_beat - event.start_beat.value();
                        } else {
                            let new_y = ctx.beat_to_y(event.end_beat) + drag_delta.y;
                            let new_beat = ctx.y_to_beat_f32(new_y);
                            // will be attached when stop dragging
                            *event.end_beat.float_mut() += new_beat - event.end_beat.value();
                        }
                    }

                    if response.drag_stopped() {
                        let from = ui.data(|data| {
                            data.get_temp::<LineEvent>(egui::Id::new("event-drag"))
                                .unwrap()
                        });
                        ui.data_mut(|data| data.remove::<LineEvent>(egui::Id::new("event-drag")));
                        if start {
                            event.start_beat = ctx.settings.attach(event.start_beat.value());
                        } else {
                            event.end_beat = ctx.settings.attach(event.end_beat.value());
                        }
                        if from != *event {
                            on_event_change(from, *event);
                        }
                    }
                };

                make_drag_zone(true);
                make_drag_zone(false);

                ui.painter().text(
                    rect.center_top(),
                    Align2::CENTER_TOP,
                    event.value.end(),
                    FontId::default(),
                    Color32::DARK_GREEN,
                );
                ui.painter().text(
                    rect.center_bottom(),
                    Align2::CENTER_BOTTOM,
                    event.value.start(),
                    FontId::default(),
                    Color32::DARK_GREEN,
                );
            }

            if response.clicked() {
                select_events.send(SelectEvent(vec![entity]));
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

        // lane
        // [0.2, 0.4, 0.6, 0.8]
        let lane_percents = iter::repeat(0.0)
            .take(5 - 1)
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
    }

    fn on_drag_selection(&self, world: &mut World, viewport: Rect, selection: Rect) -> Vec<Entity> {
        let line_entity = self.line_entity(world);

        let x_range = selection.x_range();
        let time_range = selection.y_range();

        let mut state: SystemState<(Query<(&LineEvent, &Parent, Entity)>, Res<BpmList>)> =
            SystemState::new(world);
        let (event_query, bpm_list) = state.get_mut(world);

        event_query
            .iter()
            .filter(|x| x.1.get() == line_entity)
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
}
