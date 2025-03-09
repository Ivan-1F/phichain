use crate::action::ActionRegistry;
use crate::editing::command::line::{CreateLine, MoveLineAsChild, RemoveLine};
use crate::editing::command::{CommandSequence, EditorCommand};
use crate::editing::DoCommandEvent;
use crate::selection::SelectedLine;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use egui::{Color32, Layout, RichText, Sense, Ui};
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
        let mut query = self
            .world
            .query_filtered::<Entity, (Without<Parent>, With<Line>)>();
        let mut entities = query.iter(self.world).collect::<Vec<_>>();
        entities.sort();

        let mut create_line = false;

        ui.with_layout(Layout::top_down_justified(egui::Align::Center), |ui| {
            if ui.button(t!("tab.line_list.create_line")).clicked() {
                create_line = true;
            }
        });

        ui.columns_const(|[_, state]| {
            state.style_mut().spacing.item_spacing = egui::Vec2::ZERO;

            state.columns_const(|[note_event, x_y, op_rot, spd]| {
                note_event.vertical_centered(|ui| {
                    ui.add(trunc_label!(RichText::new(t!("tab.line_list.note")).small()));
                    ui.add(trunc_label!(
                        RichText::new(t!("tab.line_list.event")).small()
                    ));
                });
                x_y.vertical_centered(|ui| {
                    ui.add(trunc_label!(RichText::new("X").small()));
                    ui.add(trunc_label!(RichText::new("Y").small()));
                });
                op_rot.vertical_centered(|ui| {
                    ui.add(trunc_label!(
                        RichText::new(t!("tab.line_list.opacity")).small()
                    ));
                    ui.add(trunc_label!(
                        RichText::new(t!("tab.line_list.rotation")).small()
                    ));
                });
                spd.vertical_centered(|ui| {
                    ui.add(trunc_label!(
                        RichText::new(t!("tab.line_list.speed")).small()
                    ));
                });
            });
        });

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
        )> = SystemState::new(self.world);

        let (note_query, event_query, query, parent_query, mut selected_line, mut do_command_event) =
            state.get_mut(self.world);

        let mut add_parent: Option<Option<Entity>> = None;
        let mut add_child = false;

        if let Ok((line, children, parent, position, rotation, opacity, speed)) = query.get(entity)
        {
            let selected = selected_line.0 == entity;

            let under_selected_node = parent_query
                .iter_ancestors(entity)
                .any(|ancestor| ancestor == selected_line.0);

            ui.columns_const(|[name, state]| {
                name.horizontal(|ui| {
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
                                do_command_event.send(DoCommandEvent(
                                    EditorCommand::MoveLineAsChild(MoveLineAsChild::new(
                                        entity,
                                        Some(selected_line.0),
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
                                do_command_event.send(DoCommandEvent(
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

                state.style_mut().spacing.item_spacing = egui::Vec2::ZERO;

                state.columns_const(|[note_event, x_y, op_rot, spd]| {
                    note_event.vertical_centered(|ui| {
                        ui.add(trunc_label!(RichText::new(format!("{:.2}", notes)).small()));
                        ui.add(trunc_label!(RichText::new(format!("{:.2}", events)).small()));
                    });
                    x_y.vertical_centered(|ui| {
                        ui.add(trunc_label!(
                            RichText::new(format!("{:.2}", position.0.x)).small()
                        ));
                        ui.add(trunc_label!(
                            RichText::new(format!("{:.2}", position.0.y)).small()
                        ));
                    });
                    op_rot.vertical_centered(|ui| {
                        ui.add(trunc_label!(
                            RichText::new(format!("{:.2}", opacity.0)).small()
                        ));
                        ui.add(trunc_label!(
                            RichText::new(format!("{:.2}", rotation.0)).small()
                        ));
                    });
                    spd.vertical_centered(|ui| {
                        ui.add(trunc_label!(
                            RichText::new(format!("{:.2}", speed.0)).small()
                        ));
                    });
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
