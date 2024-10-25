use crate::editing::command::line::{CreateLine, MoveLineAsChild, RemoveLine};
use crate::editing::command::EditorCommand;
use crate::editing::DoCommandEvent;
use crate::selection::SelectedLine;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use egui::{Color32, Layout, Sense, Ui};
use phichain_chart::event::LineEvent;
use phichain_chart::line::{Line, LineOpacity, LinePosition, LineRotation, LineSpeed};
use phichain_chart::note::Note;

struct LineList<'w> {
    world: &'w mut World,
}

macro_rules! trunc_label {
    ($text: expr) => {
        egui::Label::new($text).truncate(true)
    };
}

impl LineList<'_> {
    fn show(&mut self, ui: &mut Ui) {
        let mut state: SystemState<(
            Query<Entity, (Without<Parent>, With<Line>)>,
            EventWriter<DoCommandEvent>,
        )> = SystemState::new(self.world);
        let (root_query, mut do_command_event) = state.get_mut(self.world);
        let mut entities = root_query.iter().collect::<Vec<_>>();
        entities.sort();

        ui.with_layout(Layout::top_down_justified(egui::Align::Center), |ui| {
            if ui.button(t!("tab.line_list.create_line")).clicked() {
                do_command_event.send(DoCommandEvent(EditorCommand::CreateLine(CreateLine::new())));
            }
        });

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

        ui.separator();

        for entity in entities {
            self.entity_ui(ui, entity, 0);
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
            ResMut<SelectedLine>,
            EventWriter<DoCommandEvent>,
        )> = SystemState::new(self.world);

        let (note_query, event_query, query, mut selected_line, mut do_command_event) =
            state.get_mut(self.world);

        if let Ok((line, children, parent, position, rotation, opacity, speed)) = query.get(entity)
        {
            let selected = selected_line.0 == entity;

            ui.horizontal(|ui| {
                ui.add_space(level as f32 * 10.0);

                let mut text = egui::RichText::new(&line.name);
                if selected {
                    text = text.color(Color32::LIGHT_GREEN);
                }
                let response = ui
                    .add(egui::Label::new(text).truncate(true).sense(Sense::click()))
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
                        #[allow(clippy::collapsible_if)]
                        if parent.is_some() {
                            if ui
                                .button(t!("tab.line_list.hierarchy.move_to_root"))
                                .clicked()
                            {
                                do_command_event.send(DoCommandEvent(
                                    EditorCommand::MoveLineAsChild(MoveLineAsChild::new(
                                        entity, None,
                                    )),
                                ));
                                ui.close_menu();
                            }
                        }
                        ui.separator();
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
    }
}

pub fn line_list_tab(In(mut ui): In<Ui>, world: &mut World) {
    LineList { world }.show(&mut ui);
}
