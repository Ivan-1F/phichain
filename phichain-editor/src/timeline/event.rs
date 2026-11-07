use crate::constants::{INDICATOR_POSITION, PENDING_OPACITY};
use crate::editing::history::Edits;
use crate::selection::Selected;
use crate::timeline::click::TimelineClick;
use crate::timeline::container::TimelineRect;
use crate::timeline::pending::PendingOf;
use crate::timeline::target::{target_name, TimelineOf, TimelineTargets};
use crate::timeline::{TimelineContext, LANE_LINE_OPACITY};
use crate::timing::{ChartTime, SeekTo};
use crate::ui::widgets::beat_range_drag_zone::BeatRangeDragZone;
use bevy::prelude::{Component, Entity, In, MessageWriter, Query, Res};
use egui::{Align2, Color32, FontId, Rect, Sense, Stroke, StrokeKind, Ui};
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
use phichain_chart::line::Line;
use phichain_game::event::{EventIndex, EventOf, Events};
use std::collections::HashMap;
use std::iter;

#[derive(Component, Debug, Clone, Copy)]
pub struct EventTimeline;

#[derive(Debug, Clone)]
struct EventTrackData<T> {
    x: T,
    y: T,
    rotation: T,
    opacity: T,
    speed: T,
}

impl<T> EventTrackData<T> {
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

#[derive(Debug, Clone, Copy)]
struct Endpoint {
    y: f32,
    beat: Beat,
    value: f32,
    color: Color32,
}

/// A start or end value of an event, or the value on the seam of two continuous events
struct ValueLabel {
    kind: LineEventKind,
    pos: egui::Pos2,
    anchor: Align2,
    value: f32,
    seam: bool,
}

/// Width of an event relative to its track
const EVENT_WIDTH: f32 = 0.8;
const EVENT_FILL_OPACITY: f32 = 0.25;
const EVENT_VALUE_SIZE: f32 = 11.0;
const TRACK_HEADER_HEIGHT: f32 = 20.0;
const TRACK_HEADER_SIZE: f32 = 13.0;

/// Round to at most two decimals, dropping trailing zeros and the sign of zero
fn format_value(value: f32) -> String {
    ((value * 100.0).round() / 100.0 + 0.0).to_string()
}

pub fn event_timeline_ui(
    In((mut ui, timeline)): In<(Ui, Entity)>,
    targets: TimelineTargets,
    ctx: TimelineContext,
    event_query: Query<(&LineEvent, Entity, Option<&Selected>, Option<&PendingOf>)>,
    events_query: Query<&Events>,
    bpm_list: Res<BpmList>,
    mut clicks: MessageWriter<TimelineClick>,
    mut seek_to: MessageWriter<SeekTo>,
    mut edits: Edits,
    chart_time: Res<ChartTime>,
    index: Res<EventIndex>,
) {
    let Some(line_entity) = targets.get(timeline) else {
        return;
    };
    let viewport = ui.max_rect();
    let ui = &mut ui;

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
            Color32::WHITE.gamma_multiply(LANE_LINE_OPACITY),
        );
    }

    let track_width = viewport.width() / 5.0;
    let event_width = track_width * EVENT_WIDTH;

    let compute_x =
        |track: u8| -> f32 { track_width * track as f32 - track_width / 2.0 + viewport.min.x };

    let track_rect = |kind: LineEventKind| {
        Rect::from_center_size(
            egui::Pos2::new(compute_x(kind.into()), viewport.center().y),
            egui::Vec2::new(track_width, viewport.height()),
        )
    };

    let get_event_rect = |event: &LineEvent| {
        let x = compute_x(event.kind.into());
        let y = ctx.time_to_y(bpm_list.time_at(event.start_beat));

        let size = egui::Vec2::new(
            event_width,
            y - ctx.time_to_y(bpm_list.time_at(event.end_beat)),
        );

        let center = egui::Pos2::new(x, y - size.y / 2.0);

        Rect::from_center_size(center, size)
    };

    let Ok(events) = events_query.get(line_entity) else {
        return;
    };
    let events = events
        .iter()
        .filter_map(|entity| event_query.get(*entity).ok())
        .filter(|(_, _, _, pending)| !pending.is_some_and(|owner| owner.0 != timeline))
        .collect::<Vec<_>>();

    // used to find seams where an event continues the previous one with the same value
    let starts = events
        .iter()
        .map(|(event, ..)| ((event.kind, event.start_beat), event.value.start()))
        .collect::<HashMap<_, _>>();
    let ends = events
        .iter()
        .map(|(event, ..)| ((event.kind, event.end_beat), event.value.end()))
        .collect::<HashMap<_, _>>();

    // the nearest event endpoint beyond each edge of each track
    let mut above = EventTrackData::splat(None::<Endpoint>);
    let mut below = EventTrackData::splat(None::<Endpoint>);

    // painted after all events so no event covers them
    let mut labels = Vec::<ValueLabel>::new();

    let viewport_margin = 100.0;
    let font = FontId::monospace(EVENT_VALUE_SIZE);
    let in_view = |y: f32| viewport.y_range().contains(y);

    for (event, entity, selected, pending) in events {
        let rect = get_event_rect(event);

        let mut color = if selected.is_some() {
            Color32::LIGHT_GREEN
        } else {
            match event.value {
                LineEventValue::Transition { .. } => Color32::LIGHT_BLUE,
                LineEventValue::Constant { .. } => Color32::LIGHT_RED,
            }
        };

        if pending.is_some() {
            color = color.gamma_multiply(PENDING_OPACITY);
        }

        for endpoint in [
            Endpoint {
                y: rect.bottom(),
                beat: event.start_beat,
                value: event.value.start(),
                color,
            },
            Endpoint {
                y: rect.top(),
                beat: event.end_beat,
                value: event.value.end(),
                color,
            },
        ] {
            if endpoint.y < viewport.top() {
                let nearest = above.get_mut(event.kind);
                if nearest.is_none_or(|nearest| endpoint.y > nearest.y) {
                    *nearest = Some(endpoint);
                }
            } else if endpoint.y > viewport.bottom() {
                let nearest = below.get_mut(event.kind);
                if nearest.is_none_or(|nearest| endpoint.y < nearest.y) {
                    *nearest = Some(endpoint);
                }
            }
        }

        if rect.bottom() < viewport.top() - viewport_margin
            || rect.top() > viewport.bottom() + viewport_margin
        {
            continue;
        }

        let response = ui.allocate_rect(rect, Sense::click());
        if ui.is_rect_visible(rect) {
            ui.painter().rect(
                rect,
                0.0,
                color.gamma_multiply(EVENT_FILL_OPACITY),
                Stroke::new(1.0_f32, color),
                StrokeKind::Middle,
            );
        }

        let (start, end) = (event.value.start(), event.value.end());
        if in_view(rect.bottom()) {
            let seam = ends.get(&(event.kind, event.start_beat)) == Some(&start);
            labels.push(ValueLabel {
                kind: event.kind,
                pos: rect.center_bottom(),
                anchor: if seam {
                    Align2::CENTER_CENTER
                } else {
                    Align2::CENTER_BOTTOM
                },
                value: start,
                seam,
            });
        }
        // a seam shared with the next event is labeled by that event
        if in_view(rect.top()) && starts.get(&(event.kind, event.end_beat)) != Some(&end) {
            labels.push(ValueLabel {
                kind: event.kind,
                pos: rect.center_top(),
                anchor: Align2::CENTER_TOP,
                value: end,
                seam: false,
            });
        }

        if pending.is_none() {
            edits
                .component(entity, event, t!("history.edit_events", count = 1))
                .edit(ui, "event_range", |ui, event| {
                    ui.add(BeatRangeDragZone::new(rect, &ctx, event));
                });
        }

        if response.clicked() {
            clicks.write(TimelineClick { timeline, entity });
        }
    }

    for label in labels {
        let painter = ui.painter().with_clip_rect(track_rect(label.kind));
        let galley =
            painter.layout_no_wrap(format_value(label.value), font.clone(), Color32::WHITE);
        let rect = label.anchor.anchor_size(label.pos, galley.size());
        if label.seam {
            // cut the seam so the shared value reads as sitting between the two events
            painter.rect_filled(rect.expand2(egui::vec2(2.0, 0.0)), 0.0, Color32::BLACK);
        }
        painter.galley(rect.min, galley, Color32::WHITE);
    }

    for (edges, top) in [(&above, true), (&below, false)] {
        for (kind, endpoint) in edges.iter() {
            let Some(endpoint) = endpoint else {
                continue;
            };
            let (anchor, y) = if top {
                (Align2::CENTER_TOP, viewport.top() + 2.0)
            } else {
                (Align2::CENTER_BOTTOM, viewport.bottom() - 2.0)
            };
            let painter = ui.painter().with_clip_rect(track_rect(kind));
            let galley = painter.layout_no_wrap(
                format_value(endpoint.value),
                font.clone(),
                Color32::PLACEHOLDER,
            );
            let rect =
                anchor.anchor_size(egui::Pos2::new(compute_x(kind.into()), y), galley.size());
            let response = ui
                .allocate_rect(rect, Sense::click())
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .on_hover_text(t!("tab.timeline.event.jump_to_event"));
            let color = if response.hovered() {
                Color32::WHITE
            } else {
                endpoint.color
            };
            painter.galley(rect.min, galley, color);
            if response.clicked() {
                seek_to.write(SeekTo(bpm_list.time_at(endpoint.beat)));
            }
        }
    }

    // track headers right below the indicator, with the current values of the line under them
    let header_top = viewport.top() + viewport.height() * INDICATOR_POSITION + 1.0;
    let header_y = header_top + TRACK_HEADER_HEIGHT / 2.0;
    for (kind, txt) in [
        (LineEventKind::X, "X"),
        (LineEventKind::Y, "Y"),
        (
            LineEventKind::Rotation,
            egui_phosphor::regular::ARROWS_CLOCKWISE,
        ),
        (LineEventKind::Opacity, egui_phosphor::regular::CIRCLE_HALF),
        (LineEventKind::Speed, egui_phosphor::regular::GAUGE),
    ] {
        ui.painter().with_clip_rect(track_rect(kind)).text(
            egui::Pos2::new(compute_x(kind.into()), header_y),
            Align2::CENTER_CENTER,
            txt,
            FontId::proportional(TRACK_HEADER_SIZE),
            ui.visuals().weak_text_color(),
        );
    }
    // evaluated the same way the line itself is
    let beat: f32 = bpm_list.beat_at(chart_time.0).into();
    for kind in [
        LineEventKind::X,
        LineEventKind::Y,
        LineEventKind::Rotation,
        LineEventKind::Opacity,
        LineEventKind::Speed,
    ] {
        let Some(value) = index.evaluate(line_entity, kind, beat).value() else {
            continue;
        };
        let painter = ui.painter().with_clip_rect(track_rect(kind));
        let galley = painter.layout_no_wrap(format_value(value), font.clone(), Color32::WHITE);
        let rect = Align2::CENTER_TOP.anchor_size(
            egui::Pos2::new(compute_x(kind.into()), header_top + TRACK_HEADER_HEIGHT),
            galley.size(),
        );
        // cover the event values passing below so the two never mix
        painter.rect_filled(rect.expand2(egui::vec2(2.0, 0.0)), 0.0, Color32::BLACK);
        painter.galley(rect.min, galley, Color32::WHITE);
    }
}

pub fn event_timeline_drag_selection(
    In((timeline, selection)): In<(Entity, Rect)>,
    targets: TimelineTargets,
    rects: Query<&TimelineRect>,
    event_query: Query<(&LineEvent, &EventOf, Entity)>,
    bpm_list: Res<BpmList>,
) -> Vec<Entity> {
    let (Some(line_entity), Ok(TimelineRect(viewport))) =
        (targets.get(timeline), rects.get(timeline))
    else {
        return vec![];
    };
    let x_range = selection.x_range();
    let time_range = selection.y_range();

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

pub fn event_timeline_name(
    In(timeline): In<Entity>,
    timeline_of: Query<&TimelineOf>,
    lines: Query<&Line>,
) -> String {
    target_name(
        egui_phosphor::regular::DIAMOND,
        timeline,
        &timeline_of,
        &lines,
    )
}
