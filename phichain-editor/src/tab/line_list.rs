use crate::editing::command::line::{CreateLine, MoveLineAsChild, RemoveLine};
use crate::editing::command::{CommandSequence, EditorCommand};
use crate::editing::DoCommandEvent;
use crate::selection::SelectedLine;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use egui::{Color32, Layout, RichText, Sense, Stroke, StrokeKind, Ui};
use phichain_chart::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};
use phichain_chart::line::{
    Line, LineOpacity, LinePosition, LineRotation, LineSpeed, LineTimestamp,
};
use phichain_chart::note::Note;
use phichain_game::event::Events;

const LINE_STATE_COLUMN_WIDTH: f32 = 140.0;
const LINE_PREVIEW_COLUMN_WIDTH: f32 = 100.0;

const PREVIEW_HEIGHT: f32 = 36.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineListView {
    State,
    Preview,
}

struct LineList<'w, 's> {
    params: LineListParams<'w, 's>,
    index: usize,
}

macro_rules! trunc_label {
    ($text: expr) => {
        egui::Label::new(egui::RichText::new($text).small()).truncate()
    };
}

#[derive(SystemParam)]
pub struct LineListParams<'w, 's> {
    commands: Commands<'w, 's>,
    note_query: Query<'w, 's, &'static Note>,
    root_line_query:
        Query<'w, 's, (Entity, &'static LineTimestamp), (Without<ChildOf>, With<Line>)>,
    line_query: Query<
        'w,
        's,
        (
            &'static Line,
            Option<&'static Children>,
            &'static Events,
            Option<&'static ChildOf>,
            &'static LinePosition,
            &'static LineRotation,
            &'static LineOpacity,
            &'static LineSpeed,
        ),
    >,
    child_of_query: Query<'w, 's, &'static ChildOf>,
    selected_line: ResMut<'w, SelectedLine>,
    do_command_event: EventWriter<'w, DoCommandEvent>,
}

impl<'w, 's> LineList<'w, 's> {
    fn new(params: LineListParams<'w, 's>) -> Self {
        Self { params, index: 0 }
    }

    fn show(&mut self, ui: &mut Ui) {
        self.index = 0;

        let mut entities = self.params.root_line_query.iter().collect::<Vec<_>>();
        entities.sort_by_key(|(_, timestamp)| **timestamp);
        let entities = entities
            .iter()
            .map(|(entity, _)| *entity)
            .collect::<Vec<_>>();

        let mut create_line = false;

        ui.with_layout(Layout::top_down_justified(egui::Align::Center), |ui| {
            if ui.button(t!("tab.line_list.create_line")).clicked() {
                create_line = true;
            }
        });

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.style_mut().spacing.item_spacing = egui::Vec2::ZERO;
                ui.spacing_mut().interact_size.y = 10.0;
                let mut show = ui.data(|data| {
                    data.get_temp("line_list_view".into())
                        .unwrap_or(LineListView::State)
                });
                if ui
                    .selectable_label(
                        show == LineListView::State,
                        RichText::new(t!("tab.line_list.view.state")).small(),
                    )
                    .clicked()
                {
                    show = LineListView::State;
                }
                if ui
                    .selectable_label(
                        show == LineListView::Preview,
                        RichText::new(t!("tab.line_list.view.preview")).small(),
                    )
                    .clicked()
                {
                    show = LineListView::Preview;
                }
                ui.data_mut(|data| data.insert_temp("line_list_view".into(), show));
            });

            let view = ui.data(|data| {
                data.get_temp("line_list_view".into())
                    .unwrap_or(LineListView::State)
            });

            let width = if view == LineListView::State {
                LINE_STATE_COLUMN_WIDTH
            } else {
                LINE_PREVIEW_COLUMN_WIDTH
            };

            ui.add_space(ui.available_width() - width);

            if view == LineListView::State {
                ui.columns_const(|[note_event, x_y, op_rot, spd]| {
                    note_event.vertical_centered(|ui| {
                        ui.add(trunc_label!(t!("tab.line_list.note")));
                        ui.add(trunc_label!(t!("tab.line_list.event")));
                    });
                    x_y.vertical_centered(|ui| {
                        ui.add(trunc_label!("X"));
                        ui.add(trunc_label!("Y"));
                    });
                    op_rot.vertical_centered(|ui| {
                        ui.add(trunc_label!(t!("tab.line_list.opacity")));
                        ui.add(trunc_label!(t!("tab.line_list.rotation")));
                    });
                    spd.vertical_centered(|ui| {
                        ui.add(trunc_label!(t!("tab.line_list.speed")));
                        ui.add(trunc_label!(t!("tab.line_list.index")));
                    });
                });
            } else {
                ui.columns_const(|[position, opacity]| {
                    position.centered_and_justified(|ui| {
                        ui.label(t!("tab.line_list.preview.position"));
                    });
                    opacity.centered_and_justified(|ui| {
                        ui.label(t!("tab.line_list.preview.opacity"));
                    });
                });
            }
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for entity in entities {
                self.entity_ui(ui, entity, 0);
            }
        });

        if create_line {
            // self.world
            //     .resource_scope(|world, mut actions: Mut<ActionRegistry>| {
            //         actions.run_action(world, "phichain.create_line");
            //     });
        }
    }

    fn entity_ui(&mut self, ui: &mut Ui, entity: Entity, level: u32) {
        self.index += 1;

        let mut add_parent: Option<Option<Entity>> = None;
        let mut add_child = false;

        ui.label(format!("{:?}", self.params.line_query.get(entity).is_ok()));

        if let Ok((line, children, events, parent, position, rotation, opacity, speed)) =
            self.params.line_query.get(entity)
        {
            let selected = self.params.selected_line.0 == entity;

            let under_selected_node = self
                .params
                .child_of_query
                .iter_ancestors(entity)
                .any(|ancestor| ancestor == self.params.selected_line.0);

            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.add_space(level as f32 * 10.0);

                    let mut text = RichText::new(&line.name);
                    if selected {
                        text = text.color(Color32::LIGHT_GREEN);
                    }
                    let response = ui
                        .add(egui::Label::new(text).truncate().sense(Sense::click()))
                        .on_hover_cursor(egui::CursorIcon::PointingHand);

                    response.context_menu(|ui| {
                        ui.add_enabled_ui(!selected, |ui| {
                            if ui
                                .button(t!("tab.line_list.hierarchy.as_child_of_current_line"))
                                .clicked()
                            {
                                self.params.do_command_event.write(DoCommandEvent(
                                    EditorCommand::MoveLineAsChild(MoveLineAsChild::new(
                                        entity,
                                        Some(self.params.selected_line.0),
                                    )),
                                ));
                                ui.close_menu();
                            }
                        });
                        #[allow(clippy::collapsible_if)]
                        if parent.is_some() {
                            if ui
                                .button(t!("tab.line_list.hierarchy.move_to_root"))
                                .clicked()
                            {
                                self.params.do_command_event.write(DoCommandEvent(
                                    EditorCommand::MoveLineAsChild(MoveLineAsChild::new(
                                        entity, None,
                                    )),
                                ));
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        if ui
                            .button(t!("tab.line_list.hierarchy.add_parent"))
                            .clicked()
                        {
                            add_parent.replace(parent.map(|x| x.parent()));
                            ui.close_menu();
                        }
                        if ui.button(t!("tab.line_list.hierarchy.add_child")).clicked() {
                            add_child = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.add_enabled_ui(!under_selected_node && !selected, |ui| {
                            if ui.button(t!("tab.line_list.remove")).clicked() {
                                self.params.do_command_event.write(DoCommandEvent(
                                    EditorCommand::RemoveLine(RemoveLine::new(entity)),
                                ));
                                ui.close_menu();
                            }
                        });
                    });

                    if response.clicked() {
                        self.params.selected_line.0 = entity;
                    }
                });

                let notes = children
                    .map(|children| {
                        children
                            .iter()
                            .filter(|child| self.params.note_query.get(*child).is_ok())
                            .collect::<Vec<_>>()
                            .len()
                    })
                    .unwrap_or(0);
                let events = events.len();

                let view = ui.data(|data| {
                    data.get_temp("line_list_view".into())
                        .unwrap_or(LineListView::State)
                });

                let width = if view == LineListView::State {
                    LINE_STATE_COLUMN_WIDTH
                } else {
                    LINE_PREVIEW_COLUMN_WIDTH
                };

                ui.add_space(ui.available_width() - width);

                let prev_item_spacing = ui.style().spacing.item_spacing;

                ui.style_mut().spacing.item_spacing = egui::Vec2::ZERO;

                if view == LineListView::State {
                    ui.columns_const(|[note_event, x_y, op_rot, spd]| {
                        note_event.vertical_centered(|ui| {
                            ui.add(trunc_label!(format!("{:.2}", notes)));
                            ui.add(trunc_label!(format!("{:.2}", events)));
                        });
                        x_y.vertical_centered(|ui| {
                            ui.add(trunc_label!(format!("{:.2}", position.0.x)));
                            ui.add(trunc_label!(format!("{:.2}", position.0.y)));
                        });
                        op_rot.vertical_centered(|ui| {
                            ui.add(trunc_label!(format!("{:.2}", opacity.0)));
                            ui.add(trunc_label!(format!("{:.2}", rotation.0)));
                        });
                        spd.vertical_centered(|ui| {
                            ui.add(trunc_label!(format!("{:.2}", speed.0)));
                            ui.label(
                                RichText::new(format!("#{}", self.index))
                                    .small()
                                    .color(Color32::WHITE),
                            );
                        });
                    });
                } else {
                    ui.columns_const(|[position_ui, opacity_ui]| {
                        let x = position.0.x / CANVAS_WIDTH + 0.5;
                        let y = 1.0 - (position.0.y / CANVAS_HEIGHT + 0.5);

                        let width = PREVIEW_HEIGHT;
                        let height = PREVIEW_HEIGHT / 3.0 * 2.0;

                        let pos = |pos: egui::Pos2, rect: egui::Rect| {
                            egui::Pos2::new(pos.x * rect.width(), pos.y * rect.height())
                                + rect.min.to_vec2()
                        };

                        position_ui.vertical_centered(|ui| {
                            let (response, painter) =
                                ui.allocate_painter(egui::Vec2::new(width, height), Sense::hover());
                            painter.rect_stroke(
                                response.rect,
                                0.0,
                                Stroke::new(1.0, Color32::WHITE),
                                StrokeKind::Middle,
                            );

                            let half = 1.5;

                            let x1 = rotation.0.cos() * half;
                            let y1 = rotation.0.sin() * half;

                            let x2 = -x1;
                            let y2 = -y1;

                            let p1 = pos(egui::Pos2::new(-x1 + x, y1 + y), response.rect);
                            let p2 = pos(egui::Pos2::new(-x2 + x, y2 + y), response.rect);
                            painter.line_segment([p1, p2], Stroke::new(1.0, Color32::WHITE));

                            painter.circle_filled(
                                pos(egui::Pos2::new(x, y), response.rect),
                                2.0,
                                Color32::YELLOW,
                            );
                        });

                        opacity_ui.vertical_centered(|ui| {
                            let (response, painter) =
                                ui.allocate_painter(egui::Vec2::new(width, height), Sense::hover());
                            painter.rect_stroke(
                                response.rect,
                                0.0,
                                Stroke::new(1.0, Color32::WHITE),
                                StrokeKind::Middle,
                            );
                            painter.rect_filled(
                                egui::Rect::from_min_max(
                                    response.rect.left_top(),
                                    response.rect.center_bottom(),
                                ),
                                0.0,
                                Color32::from_rgba_unmultiplied(
                                    255,
                                    255,
                                    255,
                                    (opacity.0 * 255.0).round() as u8,
                                ),
                            );
                            painter.rect_filled(
                                egui::Rect::from_min_max(
                                    response.rect.center_top(),
                                    response.rect.right_bottom(),
                                ),
                                0.0,
                                Color32::WHITE,
                            );
                        });
                    });
                }

                ui.style_mut().spacing.item_spacing = prev_item_spacing;
            });

            ui.separator();

            let children_lines = children
                .map(|children| {
                    children
                        .iter()
                        .filter(|x| self.params.line_query.get(*x).is_ok())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            for child in children_lines {
                self.entity_ui(ui, child, level + 1);
            }
        }

        if let Some(current_parent) = add_parent {
            let mut new_line_entity = self.params.commands.spawn_empty();

            if let Some(current_parent) = current_parent {
                new_line_entity.insert(ChildOf(current_parent));
            }

            let new_line_entity = new_line_entity.id();

            self.params
                .commands
                .send_event(DoCommandEvent(EditorCommand::CommandSequence(
                    CommandSequence(vec![
                        EditorCommand::CreateLine(CreateLine::with_target(new_line_entity)),
                        EditorCommand::MoveLineAsChild(MoveLineAsChild::new(
                            entity,
                            Some(new_line_entity),
                        )),
                    ]),
                )));
        }

        if add_child {
            let new_line_entity = self.params.commands.spawn_empty().id();
            self.params
                .commands
                .send_event(DoCommandEvent(EditorCommand::CommandSequence(
                    CommandSequence(vec![
                        EditorCommand::CreateLine(CreateLine::with_target(new_line_entity)),
                        EditorCommand::MoveLineAsChild(MoveLineAsChild::new(
                            new_line_entity,
                            Some(entity),
                        )),
                    ]),
                )));
        }
    }
}

pub fn line_list_tab(In(mut ui): In<Ui>, param: LineListParams) {
    LineList::new(param).show(&mut ui);
}
