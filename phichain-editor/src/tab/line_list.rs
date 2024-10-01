use crate::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};
use crate::editing::command::line::{CreateLine, MoveLineAsChild, RemoveLine};
use crate::editing::command::EditorCommand;
use crate::editing::DoCommandEvent;
use crate::selection::SelectedLine;
use crate::settings::EditorSettings;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use egui::{Color32, Layout, Sense, Stroke, Ui};
use phichain_chart::event::LineEvent;
use phichain_chart::line::{Line, LineOpacity, LinePosition, LineRotation, LineSpeed};
use phichain_chart::note::Note;

macro_rules! trunc_label {
    ($text: expr) => {
        egui::Label::new($text).truncate(true)
    };
}

pub fn line_list_tab(
    In(mut ui): In<Ui>,
    line_query: Query<(
        &Line,
        &Children,
        Entity,
        &LinePosition,
        &LineRotation,
        &LineOpacity,
        &LineSpeed,
    )>,
    note_query: Query<&Note>,
    event_query: Query<&LineEvent>,

    mut selected_line: ResMut<SelectedLine>,

    mut do_command_event: EventWriter<DoCommandEvent>,

    mut editor_settings: ResMut<Persistent<EditorSettings>>,
) {
    let mut lines = line_query.iter().collect::<Vec<_>>();
    lines.sort_by_key(|x| x.2);
    ui.with_layout(Layout::top_down_justified(egui::Align::Center), |ui| {
        if ui.button(t!("tab.line_list.create_line")).clicked() {
            do_command_event.send(DoCommandEvent(EditorCommand::CreateLine(CreateLine::new())));
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

    ui.separator();

    egui::ScrollArea::vertical().show(&mut ui, |ui| {
        ui.columns(5, |ui| {
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
        });

        ui.separator();

        for (line, children, entity, position, rotation, opacity, speed) in lines.iter() {
            let selected = selected_line.0 == *entity;

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

            ui.columns(3, |columns| {
                columns[0].vertical_centered(|ui| {
                    ui.add(trunc_label!(format!(
                        "{} {}",
                        t!("tab.line_list.note"),
                        notes
                    )));
                });

                let mut text = egui::RichText::new(&line.name);
                if selected {
                    text = text.color(Color32::LIGHT_GREEN);
                }
                columns[1].vertical_centered(|ui| {
                    let response = ui
                        .add(egui::Label::new(text).truncate(true).sense(Sense::click()))
                        .on_hover_cursor(egui::CursorIcon::PointingHand);

                    response.context_menu(|ui| {
                        ui.add_enabled_ui(!selected, |ui| {
                            if ui.button(t!("tab.line_list.remove")).clicked() {
                                do_command_event.send(DoCommandEvent(EditorCommand::RemoveLine(
                                    RemoveLine::new(*entity),
                                )));
                                ui.close_menu();
                            }
                            if ui.button("As child of current line").clicked() {
                                do_command_event.send(DoCommandEvent(
                                    EditorCommand::MoveLineAsChild(MoveLineAsChild::new(
                                        *entity,
                                        selected_line.0,
                                    )),
                                ));
                                ui.close_menu();
                            }
                        });
                    });

                    if response.clicked() {
                        selected_line.0 = *entity;
                    }
                });
                columns[2].vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.add(trunc_label!(format!(
                            "{} {}",
                            events,
                            t!("tab.line_list.event")
                        )));
                    });
                });
            });

            if editor_settings.ui.line_list.show_states {
                ui.columns(5, |ui| {
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
        }
    });
}
