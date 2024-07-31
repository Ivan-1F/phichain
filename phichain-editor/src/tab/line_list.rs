use crate::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};
use crate::editing::command::line::{CreateLine, RemoveLine};
use crate::editing::command::EditorCommand;
use crate::editing::DoCommandEvent;
use crate::selection::SelectedLine;
use bevy::prelude::*;
use egui::{Color32, Layout, Sense, Stroke, Ui};
use phichain_chart::event::LineEvent;
use phichain_chart::line::{Line, LineOpacity, LinePosition, LineRotation, LineSpeed};
use phichain_chart::note::Note;

pub fn line_list_tab(
    In(ui): In<&mut Ui>,
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
) {
    let mut lines = line_query.iter().collect::<Vec<_>>();
    lines.sort_by_key(|x| x.2);
    ui.with_layout(Layout::top_down_justified(egui::Align::Center), |ui| {
        if ui.button(t!("tab.line_list.create_line")).clicked() {
            do_command_event.send(DoCommandEvent(EditorCommand::CreateLine(CreateLine::new())));
        }
    });
    ui.separator();
    egui::ScrollArea::both().show(ui, |ui| {
        for (index, (line, children, entity, position, rotation, opacity, speed)) in
            lines.iter().enumerate()
        {
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

            ui.horizontal(|ui| {
                ui.label(&line.name);
                let mut checked = selected;
                if ui.checkbox(&mut checked, "").clicked() {
                    selected_line.0 = *entity;
                }

                ui.add_enabled_ui(!selected, |ui| {
                    if ui.button(" × ").clicked() {
                        do_command_event.send(DoCommandEvent(EditorCommand::RemoveLine(
                            RemoveLine::new(*entity),
                        )));
                    }
                });
            });

            ui.columns(2, |columns| {
                egui::Grid::new(format!("line-{}-props-grid-left", index))
                    .num_columns(2)
                    .show(&mut columns[0], |ui| {
                        ui.label(t!("tab.line_list.note"));
                        ui.label(format!("{}", notes));
                        ui.end_row();

                        ui.label(t!("tab.line_list.position"));
                        ui.label(format!("({:.2}, {:.2})", position.0.x, position.0.y));
                        ui.end_row();

                        ui.label(t!("tab.line_list.rotation"));
                        ui.label(format!("{:.2}", rotation.0));
                        ui.end_row();
                    });
                egui::Grid::new(format!("line-{}-props-grid-right", index))
                    .num_columns(2)
                    .show(&mut columns[1], |ui| {
                        ui.label(t!("tab.line_list.event"));
                        ui.label(format!("{}", events));
                        ui.end_row();

                        ui.label(t!("tab.line_list.opacity"));
                        ui.label(format!("{:.2}", opacity.0));
                        ui.end_row();

                        ui.label(t!("tab.line_list.speed"));
                        ui.label(format!("{:.2}", speed.0));
                        ui.end_row();
                    });
            });

            ui.horizontal(|ui| {
                let x = position.0.x / CANVAS_WIDTH + 0.5;
                let y = 1.0 - (position.0.y / CANVAS_HEIGHT + 0.5);

                let width = 40.0;
                let height = 40.0 / 3.0 * 2.0;

                let pos = |pos: egui::Pos2, rect: egui::Rect| {
                    egui::Pos2::new(pos.x * rect.width(), pos.y * rect.height())
                        + rect.min.to_vec2()
                };

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

                // Position
                let (response, painter) =
                    ui.allocate_painter(egui::Vec2::new(width, height), Sense::hover());
                painter.rect_stroke(response.rect, 0.0, Stroke::new(1.0, Color32::WHITE));

                painter.circle_filled(
                    pos(egui::Pos2::new(x, y), response.rect),
                    2.0,
                    Color32::WHITE,
                );

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

            ui.separator();
        }
    });
}
