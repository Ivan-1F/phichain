use crate::editing::command::curve_note_track::CreateCurveNoteTrack;
use crate::editing::command::note::EditNote;
use crate::editing::command::EditorCommand;
use crate::editing::pending::Pending;
use crate::editing::DoCommandEvent;
use crate::selection::{SelectEvent, Selected, SelectedLine};
use crate::tab::timeline::TimelineFilter;
use crate::timeline::{Timeline, TimelineContext};
use crate::ui::widgets::easing::EasingGraph;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy_egui::EguiUserTextures;
use egui::{Color32, Pos2, Rangef, Rect, Sense, Ui};
use phichain_assets::ImageAssets;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::constants::CANVAS_WIDTH;
use phichain_chart::note::{Note, NoteKind};
use phichain_game::curve_note_track::{CurveNote, CurveNoteTrack};
use phichain_game::highlight::Highlighted;
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
                &mut Note,
                &Parent,
                Entity,
                Option<&Highlighted>,
                Option<&Selected>,
                Option<&CurveNote>,
                Option<&Pending>,
            )>,
            Query<&Selected>,
            Query<(&mut CurveNoteTrack, &Parent, Entity)>,
            Res<BpmList>,
            Res<ImageAssets>,
            Res<Assets<Image>>,
            Res<EguiUserTextures>,
            EventWriter<SelectEvent>,
            EventWriter<DoCommandEvent>,
        )> = SystemState::new(world);

        let (
            ctx,
            mut note_query,
            selected_query,
            mut track_query,
            bpm_list,
            assets,
            images,
            textures,
            mut select_events,
            mut event_writer,
        ) = state.get_mut(world);

        // TODO: optimize
        // make sure hold is rendered below other note
        let mut notes: Vec<_> = note_query.iter_mut().collect();
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

                let size = match note.kind {
                    NoteKind::Hold { hold_beat } => egui::Vec2::new(
                        viewport.width() / 8000.0 * size.x as f32,
                        y - ctx.time_to_y(bpm_list.time_at(note.beat + hold_beat)),
                    ),
                    _ => egui::Vec2::new(
                        viewport.width() / 8000.0 * size.x as f32,
                        viewport.width() / 8000.0 * size.y as f32,
                    ),
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

        let mut start_track_note = None::<Entity>;
        let mut despawn_cnt = None::<Entity>;

        for (mut note, parent, entity, highlighted, selected, curve_note, pending) in notes {
            if !ctx.settings.note_side_filter.filter(*note) {
                continue;
            }
            if parent.get() != line_entity {
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
                        let [r, g, b, a] = Color::WHITE.with_a(100.0 / 255.0).as_rgba_u8();
                        Color32::from_rgba_unmultiplied(r, g, b, a)
                    }
                } else {
                    Color32::WHITE
                }
            );

            if curve_note.is_none() {
                response.context_menu(|ui| {
                    if ui
                        .button(t!("tab.inspector.curve_note_track.start")) // TODO: this should not be under `tab.inspector`
                        .clicked()
                    {
                        start_track_note.replace(entity);
                        ui.close_menu();
                    }
                });
            }

            if let NoteKind::Hold { .. } = note.kind {
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
                        ui.data_mut(|data| data.insert_temp(egui::Id::new("hold-drag"), *note));
                    }

                    if response.dragged() {
                        let drag_delta = response.drag_delta();

                        if start {
                            let new_y = ctx.beat_to_y(note.beat) + drag_delta.y;
                            let new_beat = ctx.y_to_beat_f32(new_y);
                            // will be attached when stop dragging
                            *note.beat.float_mut() += new_beat - note.beat.value();
                        } else {
                            let new_y = ctx.beat_to_y(note.end_beat()) + drag_delta.y;
                            let end_beat = ctx.y_to_beat_f32(new_y);
                            let hold_beat = end_beat - note.beat.value();
                            // will be attached when stop dragging
                            *note.hold_beat_mut().unwrap().float_mut() +=
                                hold_beat - note.hold_beat().unwrap().value();
                        }
                    }

                    if response.drag_stopped() {
                        let from = ui.data(|data| {
                            data.get_temp::<Note>(egui::Id::new("hold-drag")).unwrap()
                        });
                        ui.data_mut(|data| data.remove::<Note>(egui::Id::new("hold-drag")));
                        if start {
                            note.beat = ctx.settings.attach(note.beat.value());
                        } else {
                            let end_beat = ctx.settings.attach(note.end_beat().value());
                            note.set_end_beat(end_beat);
                        }
                        if from != *note {
                            event_writer.send(DoCommandEvent(EditorCommand::EditNote(
                                EditNote::new(entity, from, *note),
                            )));
                        }
                    }
                };

                make_drag_zone(true);
                make_drag_zone(false);
            }

            if response.clicked() {
                let mut handled = false;

                if curve_note.is_none() {
                    for (track, _, track_entity) in &mut track_query {
                        if selected_query.get(track_entity).is_ok()
                            && track.get_entities().is_none()
                        {
                            despawn_cnt.replace(track_entity);

                            let mut completed_track = track.clone();
                            completed_track.to(entity);

                            event_writer.send(DoCommandEvent(EditorCommand::CreateCurveNoteTrack(
                                CreateCurveNoteTrack::new(parent.get(), completed_track),
                            )));

                            handled = true;
                        }
                    }
                }

                if !handled {
                    select_events.send(SelectEvent(vec![entity]));
                }
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

        for (mut track, parent, entity) in &mut track_query {
            if parent.get() != line_entity {
                continue;
            }

            if let Some((from, to)) = track.get_entities() {
                if let (Ok(from), Ok(to)) = (
                    note_query.get(from).map(|x| x.0),
                    note_query.get(to).map(|x| x.0),
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
                    ui.put(
                        rect,
                        EasingGraph::new(&mut track.options.curve)
                            .inverse(true)
                            .mirror(from.x > to.x)
                            .color(match selected_query.get(entity) {
                                Ok(_) => Color32::LIGHT_GREEN,
                                Err(_) => Color32::WHITE,
                            }),
                    );
                }
            }
        }

        if let Some(entity) = start_track_note {
            let parent = note_query.get(entity).unwrap().1.get();
            world.entity_mut(parent).with_children(|p| {
                p.spawn((CurveNoteTrack::from(entity), Selected));
            });
            // TODO: unselect everything
        }

        if let Some(despawn_cnt) = despawn_cnt {
            world.entity_mut(despawn_cnt).despawn_recursive();
        }
    }

    fn on_drag_selection(&self, world: &mut World, viewport: Rect, selection: Rect) -> Vec<Entity> {
        let line_entity = self.line_entity(world);

        let x_range = selection.x_range();
        let time_range = selection.y_range();

        let mut state: SystemState<(Query<(&Note, &Parent, Entity)>, Res<BpmList>)> =
            SystemState::new(world);
        let (note_query, bpm_list) = state.get_mut(world);

        note_query
            .iter()
            .filter(|x| x.1.get() == line_entity)
            .filter(|x| {
                let note = x.0;
                x_range.contains((note.x / CANVAS_WIDTH + 0.5) * viewport.width())
                    && time_range.contains(bpm_list.time_at(note.beat))
            })
            .map(|x| x.2)
            .collect()
    }
}
