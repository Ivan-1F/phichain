use crate::constants::PENDING_OPACITY;
use crate::editing::create_curve_note_track::start_curve_note_track;
use crate::editing::history::Edits;
use crate::selection::Selected;
use crate::tab::timeline::TimelineFilter;
use crate::timeline::click::TimelineClick;
use crate::timeline::container::TimelineRect;
use crate::timeline::kind::target_name;
use crate::timeline::pending::PendingOf;
use crate::timeline::target::{TimelineOf, TimelineTargets};
use crate::timeline::{TimelineContext, LANE_LINE_OPACITY};
use crate::ui::widgets::beat_range_drag_zone::BeatRangeDragZone;
use crate::ui::widgets::easing::EasingGraph;
use bevy::prelude::*;
use bevy_egui::EguiUserTextures;
use egui::{Color32, Pos2, Rect, Sense, Ui};
use phichain_chart::bpm_list::BpmList;
use phichain_chart::constants::CANVAS_WIDTH;
use phichain_chart::line::Line;
use phichain_chart::note::{Note, NoteKind};
use phichain_game::curve_note_track::{CurveNote, CurveNoteTrack};
use phichain_game::highlight::Highlighted;
use std::cmp::Ordering;

#[derive(Component, Debug, Clone, Copy)]
pub struct NoteTimeline;

pub fn note_timeline_ui(
    In((mut ui, timeline)): In<(Ui, Entity)>,
    targets: TimelineTargets,
    ctx: TimelineContext,
    note_query: Query<(
        &Note,
        Entity,
        Option<&Highlighted>,
        Option<&Selected>,
        Option<&CurveNote>,
        Option<&PendingOf>,
    )>,
    selected_query: Query<&Selected>,
    track_query: Query<(CurveNoteTrack, Entity)>,
    children_query: Query<&Children>,
    bpm_list: Res<BpmList>,
    (assets, dimensions, images, textures): (
        Res<phichain_assets::EguiImageAssets>,
        Res<phichain_assets::RespackDimensions>,
        Res<Assets<Image>>,
        Res<EguiUserTextures>,
    ),
    mut clicks: MessageWriter<TimelineClick>,
    mut commands: Commands,
    mut edits: Edits,
) {
    let Some(line_entity) = targets.get(timeline) else {
        return;
    };
    let viewport = ui.max_rect();
    let ui = &mut ui;

    let line_children = children_query.get(line_entity).ok();
    let children = || {
        line_children
            .into_iter()
            .flat_map(|children| children.iter())
    };

    let viewport_margin = 100.0;
    let is_visible = |note: &Note| {
        let bottom = ctx.time_to_y(bpm_list.time_at(note.beat));
        let top = match note.kind {
            NoteKind::Hold { hold_beat } => ctx.time_to_y(bpm_list.time_at(note.beat + hold_beat)),
            _ => bottom,
        };
        top <= viewport.bottom() + viewport_margin && bottom >= viewport.top() - viewport_margin
    };

    // TODO: optimize
    // make sure hold is rendered below other note
    let mut notes: Vec<_> = children()
        .filter_map(|child| note_query.get(child).ok())
        .filter(|(note, ..)| is_visible(note))
        .collect();
    notes.sort_by(|a, b| {
        let a_is_hold = a.0.kind.is_hold();
        let b_is_hold = b.0.kind.is_hold();
        if a_is_hold && b_is_hold {
            Ordering::Equal
        } else if a_is_hold && !b_is_hold {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    macro_rules! render_note {
        (note: $note:expr, highlighted: $highlighted:expr, fake: $fake:expr, tint: $tint:expr) => {{
            let note = $note;

            let x = viewport.min.x + (note.x / CANVAS_WIDTH + 0.5) * viewport.width();
            let y = ctx.time_to_y(bpm_list.time_at(note.beat));

            let get_asset = |handle: &Handle<Image>| {
                (
                    images.get(handle).unwrap().size(),
                    textures.image_id(handle).unwrap(),
                )
            };

            let handle = match (note.kind, $highlighted) {
                (NoteKind::Tap, true) => &assets.tap_highlight,
                (NoteKind::Drag, true) => &assets.drag_highlight,
                (NoteKind::Hold { .. }, true) => &assets.hold_highlight,
                (NoteKind::Flick, true) => &assets.flick_highlight,
                (NoteKind::Tap, false) => &assets.tap,
                (NoteKind::Drag, false) => &assets.drag,
                (NoteKind::Hold { .. }, false) => &assets.hold,
                (NoteKind::Flick, false) => &assets.flick,
            };

            let (size, image) = get_asset(handle);

            let unit =
                phichain_game::scale::texel_unit(viewport.width(), dimensions.note_width, 1.0);
            let size = match note.kind {
                NoteKind::Hold { hold_beat } => egui::Vec2::new(
                    size.x as f32 * unit,
                    y - ctx.time_to_y(bpm_list.time_at(note.beat + hold_beat)),
                ),
                _ => egui::Vec2::new(size.x as f32 * unit, size.y as f32 * unit),
            };

            let center = match note.kind {
                NoteKind::Hold { hold_beat: _ } => egui::Pos2::new(x, y - size.y / 2.0),
                _ => egui::Pos2::new(x, y),
            };

            let mut tint = $tint;

            if $fake {
                tint = tint.gamma_multiply(PENDING_OPACITY);
            }

            let rect = Rect::from_center_size(center, size);

            let response = ui.put(
                rect,
                egui::Image::new((image, size))
                    .maintain_aspect_ratio(false)
                    .fit_to_exact_size(size)
                    .tint(tint)
                    .sense(Sense::click()),
            );

            (response, rect)
        }};
    }

    for (note, entity, highlighted, selected, curve_note, pending) in notes {
        if !ctx.settings.note_side_filter.filter(*note) {
            continue;
        }
        if pending.is_some_and(|owner| owner.0 != timeline) {
            continue;
        }

        let (response, rect) = render_note!(
            note: &note,
            highlighted: highlighted.is_some(),
            fake: pending.is_some(),
            tint: if selected.is_some() {
                Color32::LIGHT_GREEN
            } else if curve_note.is_some() {
                if selected_query.get(curve_note.unwrap().0).is_ok() {
                    Color32::LIGHT_GREEN
                } else {
                    Color32::WHITE.gamma_multiply(100.0 / 255.0)
                }
            } else {
                Color32::WHITE
            }
        );

        if note.kind.is_hold() && curve_note.is_none() && pending.is_none() {
            edits
                .component(entity, note, t!("history.edit_notes", count = 1))
                .edit(ui, "hold_range", |ui, note| {
                    ui.add(BeatRangeDragZone::new(rect, &ctx, note));
                });
        }

        if curve_note.is_none() && pending.is_none() {
            response.context_menu(|ui| {
                if ui
                    .button(t!("tab.inspector.curve_note_track.start"))
                    .clicked()
                {
                    start_curve_note_track(&mut commands, timeline, entity, line_entity);
                    ui.close();
                }
            });
        }
        if response.clicked() {
            clicks.write(TimelineClick { timeline, entity });
        }
    }

    for percent in ctx.settings.lane_percents() {
        ui.painter().rect_filled(
            Rect::from_center_size(
                Pos2::new(
                    viewport.min.x + viewport.width() * percent,
                    viewport.center().y,
                ),
                egui::Vec2::new(2.0, viewport.height()),
            ),
            0.0,
            if percent == 0.5 {
                Color32::GREEN.gamma_multiply(LANE_LINE_OPACITY)
            } else {
                Color32::WHITE.gamma_multiply(LANE_LINE_OPACITY)
            },
        );
    }

    for (track, entity) in children().filter_map(|child| track_query.get(child).ok()) {
        if let (Ok(from), Ok(to)) = (
            note_query.get(track.from.0).map(|x| x.0),
            note_query.get(track.to.0).map(|x| x.0),
        ) {
            let (from, to) = if from.beat < to.beat {
                (from, to)
            } else {
                (to, from)
            };

            let from_x = viewport.min.x + (from.x / CANVAS_WIDTH + 0.5) * viewport.width();
            let from_y = ctx.time_to_y(bpm_list.time_at(from.beat));
            let to_x = viewport.min.x + (to.x / CANVAS_WIDTH + 0.5) * viewport.width();
            let to_y = ctx.time_to_y(bpm_list.time_at(to.beat));
            let rect = Rect::from_two_pos(Pos2::new(from_x, from_y), Pos2::new(to_x, to_y));
            edits
                .component(entity, track.options, t!("history.edit_tracks", count = 1))
                .edit(ui, "track_curve", |ui, options| {
                    ui.add(
                        EasingGraph::new(&mut options.curve)
                            .rect(rect)
                            .inverse(true)
                            .mirror(from.x > to.x)
                            .color(match selected_query.get(entity) {
                                Ok(_) => Color32::LIGHT_GREEN,
                                Err(_) => Color32::WHITE,
                            }),
                    );
                });
        }
    }
}

pub fn note_timeline_drag_selection(
    In((timeline, selection)): In<(Entity, Rect)>,
    targets: TimelineTargets,
    rects: Query<&TimelineRect>,
    children_query: Query<&Children>,
    note_query: Query<(&Note, Entity)>,
    bpm_list: Res<BpmList>,
) -> Vec<Entity> {
    let (Some(line_entity), Ok(TimelineRect(viewport))) =
        (targets.get(timeline), rects.get(timeline))
    else {
        return vec![];
    };
    let x_range = selection.x_range();
    let time_range = selection.y_range();

    let Ok(children) = children_query.get(line_entity) else {
        return vec![];
    };

    children
        .iter()
        .filter_map(|child| note_query.get(child).ok())
        .filter(|x| {
            let note = x.0;
            x_range.contains((note.x / CANVAS_WIDTH + 0.5) * viewport.width())
                && time_range.contains(bpm_list.time_at(note.beat))
        })
        .map(|x| x.1)
        .collect()
}

pub fn note_timeline_name(
    In(timeline): In<Entity>,
    timeline_of: Query<&TimelineOf>,
    lines: Query<&Line>,
) -> String {
    target_name(
        egui_phosphor::regular::MUSIC_NOTE,
        timeline,
        &timeline_of,
        &lines,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection::SelectedLine;
    use crate::timeline::target::FollowSelectedLine;
    use bevy::ecs::system::RunSystemOnce;
    use phichain_chart::beat::Beat;

    #[test]
    fn drag_selection_only_selects_notes_of_the_target_line() {
        let mut world = World::new();
        world.init_resource::<BpmList>();
        let line = world.spawn(Line::default()).id();
        let other = world.spawn(Line::default()).id();
        world.insert_resource(SelectedLine(other));
        let note = |beat: f32| Note::new(NoteKind::Tap, true, Beat::from(beat), 0.0, 1.0);
        let inside = world.spawn((note(1.0), ChildOf(line))).id();
        world.spawn((note(10.0), ChildOf(line)));
        let on_other = world.spawn((note(1.0), ChildOf(other))).id();
        let viewport = Rect::from_min_size(Pos2::ZERO, egui::vec2(100.0, 100.0));
        let fixed = world
            .spawn((NoteTimeline, TimelineOf(line), TimelineRect(viewport)))
            .id();
        let follow = world
            .spawn((NoteTimeline, FollowSelectedLine, TimelineRect(viewport)))
            .id();
        let selection = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 1.0));

        let selected = world
            .run_system_once_with(note_timeline_drag_selection, (fixed, selection))
            .unwrap();
        assert_eq!(selected, vec![inside]);

        let selected = world
            .run_system_once_with(note_timeline_drag_selection, (follow, selection))
            .unwrap();
        assert_eq!(selected, vec![on_other]);
    }
}
