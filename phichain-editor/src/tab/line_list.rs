use crate::action::ActionRegistry;
use crate::editing::command::line::{CreateLine, MoveLineAsChild, RemoveLine};
use crate::editing::command::{CommandSequence, EditorCommand};
use crate::editing::DoCommandEvent;
use crate::selection::SelectedLine;
use crate::settings::EditorSettings;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use egui::{Color32, Layout, Sense, Stroke, Ui};
use phichain_chart::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};
use phichain_chart::event::LineEvent;
use phichain_chart::line::{Line, LineOpacity, LinePosition, LineRotation, LineSpeed};
use phichain_chart::note::Note;

struct LineList<'w> {
    world: &'w mut World,
}

macro_rules! trunc_label {
    ($text: expr) => {
        egui::Label::new($text).truncate()
    };
}

impl LineList<'_> {
    fn show(&mut self, ui: &mut Ui) {
        let mut state: SystemState<(
            Query<Entity, (Without<Parent>, With<Line>)>,
            ResMut<Persistent<EditorSettings>>,
        )> = SystemState::new(self.world);
        let (root_query, mut editor_settings) = state.get_mut(self.world);
        let mut entities = root_query.iter().collect::<Vec<_>>();
        entities.sort();

        let mut create_line = false;

        ui.with_layout(Layout::top_down_justified(egui::Align::Center), |ui| {
            if ui.button(t!("tab.line_list.create_line")).clicked() {
                create_line = true;
            }
        });

        ui.columns(2, |columns| {
            columns[0].vertical_centered(|ui| {
                if ui
                    .checkbox(
                        &mut editor_settings.ui.line_list.show_states,
                        t!("tab.line_list.show_states"),
                    )
                    .changed()
                {
                    // TODO: handle error (global)
                    let _ = editor_settings.persist();
                }
            });
            columns[1].vertical_centered(|ui| {
                if ui
                    .checkbox(
                        &mut editor_settings.ui.line_list.show_previews,
                        t!("tab.line_list.show_previews"),
                    )
                    .changed()
                {
                    // TODO: handle error (global)
                    let _ = editor_settings.persist();
                }
            });
        });

        if editor_settings.ui.line_list.show_states {
            ui.columns(7, |ui| {
                ui[0].vertical_centered(|ui| {
                    ui.add(trunc_label!("X"));
                });
                ui[1].vertical_centered(|ui| {
                    ui.add(trunc_label!("Y"));
                });
                ui[2].vertical_centered(|ui| {
                    ui.add(trunc_label!(t!("tab.line_list.rotation")));
                });
                ui[3].vertical_centered(|ui| {
                    ui.add(trunc_label!(t!("tab.line_list.opacity")));
                });
                ui[4].vertical_centered(|ui| {
                    ui.add(trunc_label!(t!("tab.line_list.speed")));
                });

                ui[5].vertical_centered(|ui| {
                    ui.add(trunc_label!(t!("tab.line_list.note")));
                });
                ui[6].vertical_centered(|ui| {
                    ui.add(trunc_label!(t!("tab.line_list.event")));
                });
            });
        }

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for entity in entities {
                self.entity_ui(ui, entity, 0);
            }
        });

        if create_line {
            self.world
                .resource_scope(|world, mut actions: Mut<ActionRegistry>| {
                    actions.run_action(world, "phichain.create_line");
                });
        }
    }

    fn entity_ui(&mut self, ui: &mut Ui, entity: Entity, level: u32) {
        let mut state: SystemState<(
            Query<&Note>,
            Query<&LineEvent>,
            Query<(
                &Line,
                &Children,
                Option<&Parent>,
                &LinePosition,
                &LineRotation,
                &LineOpacity,
                &LineSpeed,
            )>,
            Query<&Parent>,
            ResMut<SelectedLine>,
            EventWriter<DoCommandEvent>,
            Res<Persistent<EditorSettings>>,
        )> = SystemState::new(self.world);

        let (
            note_query,
            event_query,
            query,
            parent_query,
            mut selected_line,
            mut do_command_event,
            editor_settings,
        ) = state.get_mut(self.world);

        let mut add_parent: Option<Option<Entity>> = None;
        let mut add_child = false;

        if let Ok((line, children, parent, position, rotation, opacity, speed)) = query.get(entity)
        {
            let selected = selected_line.0 == entity;

            let under_selected_node = parent_query
                .iter_ancestors(entity)
                .any(|ancestor| ancestor == selected_line.0);

            ui.horizontal(|ui| {
                ui.add_space(level as f32 * 10.0);

                let mut text = egui::RichText::new(&line.name);
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
                            do_command_event.send(DoCommandEvent(EditorCommand::MoveLineAsChild(
                                MoveLineAsChild::new(entity, Some(selected_line.0)),
                            )));
                            ui.close_menu();
                        }
                    });
                    #[allow(clippy::collapsible_if)]
                    if parent.is_some() {
                        if ui
                            .button(t!("tab.line_list.hierarchy.move_to_root"))
                            .clicked()
                        {
                            do_command_event.send(DoCommandEvent(EditorCommand::MoveLineAsChild(
                                MoveLineAsChild::new(entity, None),
                            )));
                            ui.close_menu();
                        }
                    }
                    ui.separator();
                    if ui
                        .button(t!("tab.line_list.hierarchy.add_parent"))
                        .clicked()
                    {
                        add_parent.replace(parent.map(|x| x.get()));
                        ui.close_menu();
                    }
                    if ui.button(t!("tab.line_list.hierarchy.add_child")).clicked() {
                        add_child = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.add_enabled_ui(!under_selected_node && !selected, |ui| {
                        if ui.button(t!("tab.line_list.remove")).clicked() {
                            do_command_event.send(DoCommandEvent(EditorCommand::RemoveLine(
                                RemoveLine::new(entity),
                            )));
                            ui.close_menu();
                        }
                    });
                });

                if response.clicked() {
                    selected_line.0 = entity;
                }
            });

            let notes = children
                .iter()
                .filter(|child| note_query.get(**child).is_ok())
                .collect::<Vec<_>>()
                .len();
            let events = children
                .iter()
                .filter(|child| event_query.get(**child).is_ok())
                .collect::<Vec<_>>()
                .len();

            if editor_settings.ui.line_list.show_states {
                ui.columns(7, |ui| {
                    ui[0].vertical_centered(|ui| {
                        ui.add(trunc_label!(format!("{:.2}", position.0.x)));
                    });
                    ui[1].vertical_centered(|ui| {
                        ui.add(trunc_label!(format!("{:.2}", position.0.y)));
                    });
                    ui[2].vertical_centered(|ui| {
                        ui.add(trunc_label!(format!("{:.2}", rotation.0.to_degrees())));
                    });
                    ui[3].vertical_centered(|ui| {
                        ui.add(trunc_label!(format!("{:.2}", opacity.0)));
                    });
                    ui[4].vertical_centered(|ui| {
                        ui.add(trunc_label!(format!("{:.2}", speed.0)));
                    });
                    ui[5].vertical_centered(|ui| {
                        ui.add(trunc_label!(format!("{:.2}", notes)));
                    });
                    ui[6].vertical_centered(|ui| {
                        ui.add(trunc_label!(format!("{:.2}", events)));
                    });
                });
            }

            if editor_settings.ui.line_list.show_previews {
                ui.columns(4, |columns| {
                    let x = position.0.x / CANVAS_WIDTH + 0.5;
                    let y = 1.0 - (position.0.y / CANVAS_HEIGHT + 0.5);

                    let width = 40.0;
                    let height = 40.0 / 3.0 * 2.0;

                    let pos = |pos: egui::Pos2, rect: egui::Rect| {
                        egui::Pos2::new(pos.x * rect.width(), pos.y * rect.height())
                            + rect.min.to_vec2()
                    };

                    columns[0].vertical_centered(|ui| {
                        // Preview
                        let (response, painter) =
                            ui.allocate_painter(egui::Vec2::new(width, height), Sense::hover());
                        painter.rect_stroke(response.rect, 0.0, Stroke::new(1.0, Color32::WHITE));

                        let half = 1.5;

                        let x1 = rotation.0.cos() * half;
                        let y1 = rotation.0.sin() * half;

                        let x2 = -x1;
                        let y2 = -y1;

                        let p1 = pos(egui::Pos2::new(-x1 + x, y1 + y), response.rect);
                        let p2 = pos(egui::Pos2::new(-x2 + x, y2 + y), response.rect);
                        painter.line_segment(
                            [p1, p2],
                            Stroke::new(
                                1.0,
                                Color32::from_rgba_unmultiplied(
                                    255,
                                    255,
                                    255,
                                    (opacity.0 * 255.0).round() as u8,
                                ),
                            ),
                        );
                    });

                    columns[1].vertical_centered(|ui| {
                        // Position
                        let (response, painter) =
                            ui.allocate_painter(egui::Vec2::new(width, height), Sense::hover());
                        painter.rect_stroke(response.rect, 0.0, Stroke::new(1.0, Color32::WHITE));

                        painter.circle_filled(
                            pos(egui::Pos2::new(x, y), response.rect),
                            2.0,
                            Color32::WHITE,
                        );
                    });

                    columns[2].vertical_centered(|ui| {
                        // Opacity
                        let (response, painter) =
                            ui.allocate_painter(egui::Vec2::new(width, height), Sense::hover());
                        painter.rect_stroke(response.rect, 0.0, Stroke::new(1.0, Color32::WHITE));
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

                    columns[3].vertical_centered(|ui| {
                        // Rotation
                        let (response, painter) =
                            ui.allocate_painter(egui::Vec2::new(width, height), Sense::hover());
                        painter.rect_stroke(response.rect, 0.0, Stroke::new(1.0, Color32::WHITE));

                        let half = 1.5;

                        let x1 = rotation.0.cos() * half;
                        let y1 = rotation.0.sin() * half;

                        let x2 = -x1;
                        let y2 = -y1;

                        let p1 = pos(egui::Pos2::new(0.5 - x1, y1 + 0.5), response.rect);
                        let p2 = pos(egui::Pos2::new(0.5 - x2, y2 + 0.5), response.rect);
                        painter.line_segment([p1, p2], Stroke::new(1.0, Color32::WHITE));
                    });
                });
            }

            ui.separator();

            let children_lines = children
                .iter()
                .filter(|x| query.get(**x).is_ok())
                .copied()
                .collect::<Vec<_>>();
            for child in children_lines {
                self.entity_ui(ui, child, level + 1);
            }
        }

        if let Some(current_parent) = add_parent {
            let mut new_line_entity = self.world.spawn_empty();

            if let Some(current_parent) = current_parent {
                new_line_entity.set_parent(current_parent);
            }

            let new_line_entity = new_line_entity.id();

            self.world
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
            let new_line_entity = self.world.spawn_empty().id();
            self.world
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

pub fn line_list_tab(In(mut ui): In<Ui>, world: &mut World) {
    LineList { world }.show(&mut ui);
}
