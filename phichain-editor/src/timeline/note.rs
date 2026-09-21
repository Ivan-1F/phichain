use crate::editing::history::Edits;
use crate::notification::{ToastsExt, ToastsStorage};
use crate::selection::{Select, Selected, SelectedLine};
use crate::tab::timeline::TimelineFilter;
use crate::timeline::{Timeline, TimelineContext};
use crate::ui::widgets::beat_range_drag_zone::BeatRangeDragZone;
use crate::ui::widgets::easing::EasingGraph;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy_egui::EguiUserTextures;
use egui::{Color32, Pos2, Rect, Sense, Ui};
use phichain_chart::bpm_list::BpmList;
use phichain_chart::constants::CANVAS_WIDTH;
use phichain_chart::curve_note_track::CurveNoteTrackOptions;
use phichain_chart::line::Line;
use phichain_chart::note::{Note, NoteKind};
use phichain_game::curve_note_track::{
    CurveNote, CurveNoteTrack, CurveNoteTrackFrom, CurveNoteTrackTo,
};
use phichain_game::highlight::Highlighted;
use phichain_game::Pending;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct NoteTimeline(pub Option<Entity>);

impl NoteTimeline {
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

impl Timeline for NoteTimeline {
    fn ui(&self, ui: &mut Ui, world: &mut World, viewport: Rect) {
        let line_entity = self.line_entity(world);

        let mut state: SystemState<(
            TimelineContext,
            Query<(
                &Note,
                &ChildOf,
                Entity,
                Option<&Highlighted>,
                Option<&Selected>,
                Option<&CurveNote>,
                Option<&Pending>,
            )>,
            Query<&Selected>,
            Query<(CurveNoteTrack, &ChildOf, Entity)>,
            Res<BpmList>,
            Res<phichain_assets::EguiImageAssets>,
            Res<phichain_assets::RespackDimensions>,
            Res<Assets<Image>>,
            Res<EguiUserTextures>,
            MessageWriter<Select>,
            Commands,
            Edits,
            ResMut<ToastsStorage>,
            Query<
                (Entity, &CurveNoteTrackFrom, &CurveNoteTrackOptions),
                (With<Pending>, Without<CurveNoteTrackTo>),
            >,
        )> = SystemState::new(world);

        let (
            ctx,
            note_query,
            selected_query,
            track_query,
            bpm_list,
            assets,
            dimensions,
            images,
            textures,
            mut select_events,
            mut commands,
            mut edits,
            mut toasts,
            pending_tracks,
        ) = state.get_mut(world);

        // TODO: optimize
        // make sure hold is rendered below other note
        let mut notes: Vec<_> = note_query.iter().collect();
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
                    tint = Color32::from_rgba_unmultiplied(tint.r(), tint.g(), tint.b(), 20);
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

        for (note, child_of, entity, highlighted, selected, curve_note, pending) in notes {
            if !ctx.settings.note_side_filter.filter(*note) {
                continue;
            }
            if child_of.parent() != line_entity {
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
                        let [r, g, b, a] = bevy::color::palettes::css::WHITE.with_alpha(100.0 / 255.0).to_u8_array();
                        Color32::from_rgba_unmultiplied(r, g, b, a)
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
                        for (preview, _, _) in &pending_tracks {
                            commands.entity(preview).despawn();
                        }
                        commands.spawn((CurveNoteTrackFrom(entity), ChildOf(line_entity), Pending));
                        ui.close();
                    }
                });
            }
            if response.clicked() {
                if curve_note.is_none() && pending.is_none() {
                    if let Ok((preview, from, options)) = pending_tracks.single() {
                        let from = from.0;
                        if from == entity {
                            toasts.info(t!("tab.timeline.curve_note_track.same_endpoint"));
                            continue;
                        }
                        let options = options.clone();
                        let line = note_query.get(from).unwrap().1.parent();
                        if line != line_entity {
                            continue;
                        }
                        commands.entity(preview).despawn();
                        edits.once(t!("history.create_track"), move |commands| {
                            commands.spawn((
                                options,
                                CurveNoteTrackFrom(from),
                                CurveNoteTrackTo(entity),
                                ChildOf(line),
                            ));
                        });
                        continue;
                    }
                }
                select_events.write(Select(vec![entity]));
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
                    Color32::from_rgba_unmultiplied(0, 255, 0, 40)
                } else {
                    Color32::from_rgba_unmultiplied(255, 255, 255, 40)
                },
            );
        }

        for (track, child_of, entity) in &track_query {
            if child_of.parent() != line_entity {
                continue;
            }

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
        state.apply(world);
    }

    fn on_drag_selection(&self, world: &mut World, viewport: Rect, selection: Rect) -> Vec<Entity> {
        let line_entity = self.line_entity(world);

        let x_range = selection.x_range();
        let time_range = selection.y_range();

        let mut state: SystemState<(Query<(&Note, &ChildOf, Entity)>, Res<BpmList>)> =
            SystemState::new(world);
        let (note_query, bpm_list) = state.get_mut(world);

        note_query
            .iter()
            .filter(|x| x.1.parent() == line_entity)
            .filter(|x| {
                let note = x.0;
                x_range.contains((note.x / CANVAS_WIDTH + 0.5) * viewport.width())
                    && time_range.contains(bpm_list.time_at(note.beat))
            })
            .map(|x| x.2)
            .collect()
    }

    fn name(&self, world: &World) -> String {
        match self.0 {
            None => format!(
                "{} {}",
                egui_phosphor::regular::MUSIC_NOTE,
                t!("tab.timeline_setting.timelines.binding")
            ),
            Some(entity) => format!(
                "{} {}",
                egui_phosphor::regular::MUSIC_NOTE,
                world.get::<Line>(entity).unwrap().name
            ),
        }
    }
}
