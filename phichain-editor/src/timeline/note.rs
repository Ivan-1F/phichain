use crate::editing::command::note::EditNote;
use crate::editing::command::EditorCommand;
use crate::editing::fill_notes::{generate_notes, FillingNotes};
use crate::editing::pending::Pending;
use crate::editing::DoCommandEvent;
use crate::selection::{SelectEvent, Selected, SelectedLine};
use crate::tab::timeline::TimelineFilter;
use crate::timeline::{Timeline, TimelineContext};
use crate::ui::utils::draw_easing;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy_egui::EguiUserTextures;
use egui::{emath, Color32, Pos2, Rangef, Rect, Sense, Stroke, Ui};
use phichain_assets::ImageAssets;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::constants::CANVAS_WIDTH;
use phichain_chart::easing::Easing;
use phichain_chart::note::{Note, NoteKind};
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
                Option<&Pending>,
            )>,
            Query<&mut FillingNotes>,
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
            mut filling_notes_query,
            bpm_list,
            assets,
            images,
            textures,
            mut select_events,
            mut event_writer,
        ) = state.get_mut(world);

        // TODO: optimize
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

        let mut start_filling_note = None::<Entity>;

        for (mut note, parent, entity, highlighted, selected, pending) in notes {
            if !ctx.settings.note_side_filter.filter(*note) {
                continue;
            }
            if parent.get() != line_entity {
                continue;
            }

            let x = viewport.min.x + (note.x / CANVAS_WIDTH + 0.5) * viewport.width();
            let y = ctx.time_to_y(bpm_list.time_at(note.beat));

            let get_asset = |handle: &Handle<Image>| {
                (
                    images.get(handle).unwrap().size(),
                    textures.image_id(handle).unwrap(),
                )
            };

            let handle = match (note.kind, highlighted.is_some()) {
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

            let mut tint = if selected.is_some() {
                Color32::LIGHT_GREEN
            } else {
                Color32::WHITE
            };

            if pending.is_some() {
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

            response.context_menu(|ui| {
                ui.add_enabled_ui(filling_notes_query.is_empty(), |ui| {
                    if ui
                        .button(t!("tab.inspector.filling_notes.start")) // TODO: this should not be under `tab.inspector`
                        .on_disabled_hover_text(
                            "Please cancel the current filling task to start a new one",
                        )
                        .clicked()
                    {
                        start_filling_note.replace(entity);
                        ui.close_menu();
                    }
                });
            });

            if let NoteKind::Hold { .. } = note.kind {
                let mut make_drag_zone = |start: bool| {
                    let drag_zone = egui::Rect::from_x_y_ranges(
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
                if let Ok(mut filling) = filling_notes_query.get_single_mut() {
                    filling.to(entity);
                } else {
                    select_events.send(SelectEvent(vec![entity]));
                }
            }
        }

        for percent in ctx.settings.lane_percents() {
            ui.painter().rect_filled(
                Rect::from_center_size(
                    egui::Pos2::new(
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

        // TODO: optimize
        if let Ok(mut filling) = filling_notes_query.get_single_mut() {
            if let (Some(from), Some(to)) = (filling.from, filling.to) {
                let from = note_query.get(from);
                let to = note_query.get(to);
                if let (Ok(from), Ok(to)) = (from, to) {
                    let from_x =
                        viewport.min.x + (from.0.x / CANVAS_WIDTH + 0.5) * viewport.width();
                    let from_y = ctx.time_to_y(bpm_list.time_at(from.0.beat));
                    let to_x = viewport.min.x + (to.0.x / CANVAS_WIDTH + 0.5) * viewport.width();
                    let to_y = ctx.time_to_y(bpm_list.time_at(to.0.beat));
                    let rect = Rect::from_two_pos(Pos2::new(from_x, from_y), Pos2::new(to_x, to_y));
                    let response = draw_easing(ui, rect, filling.easing);
                    if let Easing::Custom(ref mut x1, ref mut y1, ref mut x2, ref mut y2) =
                        filling.easing
                    {
                        let painter = ui.painter_at(rect);
                        let to_screen = emath::RectTransform::from_to(
                            Rect::from_min_size(Pos2::ZERO, egui::Vec2::new(1.0, 1.0)),
                            rect,
                        );

                        let mut p1 = Pos2::new(*x1, 1.0 - *y1);
                        let mut p2 = Pos2::new(*x2, 1.0 - *y2);
                        let size = egui::Vec2::splat(2.0 * 4.0);

                        let point_in_screen = to_screen.transform_pos(p1);
                        let point_rect = Rect::from_center_size(point_in_screen, size);
                        let point_id = response.id.with(1);
                        egui::Id::new("");
                        let point_response = ui.interact(point_rect, point_id, Sense::drag());

                        p1 += point_response.drag_delta() / rect.size();
                        p1 = to_screen.from().clamp(p1);

                        let point_in_screen = to_screen.transform_pos(p2);
                        let point_rect = Rect::from_center_size(point_in_screen, size);
                        let point_id = response.id.with(2);
                        let point_response = ui.interact(point_rect, point_id, Sense::drag());

                        p2 += point_response.drag_delta() / rect.size();
                        p2 = to_screen.from().clamp(p2);

                        ui.add_space(4.0); // add some space to make sure 0, 0 and drag values are not too close

                        if p1.x != *x1 || p1.y != *y1 || p2.x != *x2 || p2.y != *y2 {
                            filling.easing = Easing::Custom(p1.x, 1.0 - p1.y, p2.x, 1.0 - p2.y);
                        }

                        painter.circle(to_screen * p1, 4.0, Color32::WHITE, Stroke::NONE);
                        painter.circle(to_screen * p2, 4.0, Color32::WHITE, Stroke::NONE);

                        painter.line_segment(
                            [to_screen * Pos2::new(0.0, 1.0), to_screen * p1],
                            Stroke::new(2.0, Color32::GRAY),
                        );
                        painter.line_segment(
                            [to_screen * Pos2::new(1.0, 0.0), to_screen * p2],
                            Stroke::new(2.0, Color32::GRAY),
                        );
                    }

                    for note in generate_notes(*from.0, *to.0, &filling) {
                        let x = viewport.min.x + (note.x / CANVAS_WIDTH + 0.5) * viewport.width();
                        let y = ctx.time_to_y(bpm_list.time_at(note.beat));

                        let get_asset = |handle: &Handle<Image>| {
                            (
                                images.get(handle).unwrap().size(),
                                textures.image_id(handle).unwrap(),
                            )
                        };

                        let (size, image) = get_asset(match note.kind {
                            NoteKind::Tap => &assets.tap,
                            NoteKind::Drag => &assets.drag,
                            NoteKind::Flick => &assets.flick,
                            NoteKind::Hold { .. } => &assets.hold,
                        });

                        let size = egui::Vec2::new(
                            viewport.width() / 8000.0 * size.x as f32,
                            viewport.width() / 8000.0 * size.y as f32,
                        );

                        let center = Pos2::new(x, y);

                        let tint = Color32::from_rgba_unmultiplied(255, 255, 255, 100);

                        let rect = Rect::from_center_size(center, size);

                        ui.put(
                            rect,
                            egui::Image::new((image, size))
                                .maintain_aspect_ratio(false)
                                .fit_to_exact_size(size)
                                .tint(tint)
                                .sense(Sense::click()),
                        );
                    }
                }
            }
        }

        if let Some(entity) = start_filling_note {
            world.spawn(FillingNotes::from(entity));
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
