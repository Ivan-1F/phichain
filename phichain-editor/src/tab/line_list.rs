use crate::editing::command::line::{CreateLine, MoveLineAsChild, RemoveLine};
use crate::editing::command::EditorCommand;
use crate::editing::DoCommandEvent;
use crate::selection::SelectedLine;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use egui::{vec2, Color32, Layout, Sense, Ui};
use phichain_chart::event::LineEvent;
use phichain_chart::line::Line;
use phichain_chart::note::Note;

struct LineList<'w> {
    world: &'w mut World,
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

        for entity in entities {
            self.entity_ui(ui, entity, 0);
        }
    }

    fn entity_ui(&mut self, ui: &mut Ui, entity: Entity, level: u32) {
        let mut state: SystemState<(
            Query<&Note>,
            Query<&LineEvent>,
            Query<(&Line, &Children, Option<&Parent>)>,
            ResMut<SelectedLine>,
            EventWriter<DoCommandEvent>,
        )> = SystemState::new(self.world);

        let (note_query, event_query, query, mut selected_line, mut do_command_event) =
            state.get_mut(self.world);

        if let Ok((line, children, parent)) = query.get(entity) {
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
                        if ui.button("As child of current line").clicked() {
                            do_command_event.send(DoCommandEvent(EditorCommand::MoveLineAsChild(
                                MoveLineAsChild::new(entity, Some(selected_line.0)),
                            )));
                            ui.close_menu();
                        }
                        #[allow(clippy::collapsible_if)]
                        if parent.is_some() {
                            if ui.button("Move to root").clicked() {
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

                let space = ui.available_width() - 100.0;
                if space > 0.0 {
                    ui.add_space(space);
                }
                ui.add_sized(vec2(40.0, 18.0), egui::Label::new(format!("N:{}", notes)));
                ui.add_sized(vec2(40.0, 18.0), egui::Label::new(format!("E:{}", events)));
            });

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
