use crate::action::{ActionRegistrationExt, ActionRegistry, RunActionEvent};
use crate::hotkey::modifier::Modifier;
use crate::hotkey::{Hotkey, HotkeyRegistry};
use bevy::app::App;
use bevy::prelude::{
    Commands, Component, Entity, EventWriter, IntoSystemConfigs, KeyCode, Plugin, Query, Res,
    Update, Window, With, World,
};
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContext;
use egui::{Align2, TextEdit, Widget};
use phichain_game::GameSet;

pub struct ActionPanelPlugin;

impl Plugin for ActionPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_action(
            "phichain.open_action_panel",
            open_action_panel_system,
            Some(Hotkey::new(KeyCode::KeyK, vec![Modifier::Control])),
        )
        .add_systems(Update, action_panel_ui_system.in_set(GameSet));
    }
}

#[derive(Clone, Debug, Default, Component)]
pub struct ActionPanel {
    query: String,
    cursor: Option<usize>,
}

// TODO: using exclusive system here because somehow `mut Commands` does not work with `world.run_system()`
fn open_action_panel_system(world: &mut World) {
    if world.query::<&ActionPanel>().get_single(world).is_ok() {
        return;
    }

    world.spawn(ActionPanel::default());
}

fn action_panel_ui_system(
    mut commands: Commands,
    actions: Res<ActionRegistry>,
    hotkeys: Res<HotkeyRegistry>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut context: Query<&mut EguiContext>,
    mut query: Query<(Entity, &mut ActionPanel)>,

    mut run: EventWriter<RunActionEvent>,
) {
    let Ok((entity, mut panel)) = query.get_single_mut() else {
        return;
    };

    let Ok(egui_context) = context.get_single_mut() else {
        return;
    };
    let mut egui_context = egui_context.clone();
    let ctx = egui_context.get_mut();

    let window = window.single();

    let response = egui::Window::new("Action Panel")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .fixed_size((window.width() * 0.6, window.height() * 0.5))
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.style_mut().interaction.selectable_labels = false;

            TextEdit::singleline(&mut panel.query)
                .desired_width(f32::INFINITY)
                .ui(ui)
                .request_focus();

            ui.separator();

            let entries = actions
                .0
                .iter()
                .filter(|(id, _)| id.to_string() != "phichain.open_action_panel")
                .filter(|(id, _)| {
                    let label = t!(format!("action.{}", id).as_str()).to_string();
                    label.contains(panel.query.as_str())
                        || id.to_string().contains(panel.query.as_str())
                })
                .collect::<Vec<_>>();

            for (index, (id, action)) in entries.iter().enumerate() {
                let selected = panel.cursor.is_some_and(|x| x == index);

                egui::Frame::none()
                    .fill(if selected {
                        egui::Color32::from_rgba_unmultiplied(64, 94, 168, 100)
                    } else {
                        egui::Color32::TRANSPARENT
                    })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(t!(format!("action.{}", id).as_str()));

                            let remain = ui.available_width();
                            ui.add_space(remain - 200.0);

                            if action.enable_hotkey {
                                if let Some(hotkey) = hotkeys.0.get(*id) {
                                    ui.label(hotkey.to_string());
                                }
                            }

                            ui.add_space(ui.available_width());
                        });
                    });
            }

            ui.style_mut().interaction.selectable_labels = true;

            ui.input(|x| {
                if x.key_pressed(egui::Key::Escape) {
                    commands.entity(entity).despawn();
                }
            });

            // Update cursor

            // 1. If the cursor is `Some` and exceeds the number of entries, update it to `Some(0)` if there is at least one entry present. Otherwise, set it to `None`
            if panel.cursor.is_some_and(|x| x >= entries.len()) {
                if !entries.is_empty() {
                    panel.cursor = Some(0);
                } else {
                    panel.cursor = None;
                }
            }

            // 2. if cursor is None, but at least 1 entry present, update it to Some(0)
            if panel.cursor.is_none() && !entries.is_empty() {
                panel.cursor = Some(0);
            }

            // 3. move cursor with up and down arrows
            ui.input(|x| {
                if let Some(cursor) = panel.cursor {
                    if x.key_pressed(egui::Key::ArrowUp) {
                        panel.cursor = Some(cursor.saturating_sub(1));
                    } else if x.key_pressed(egui::Key::ArrowDown) {
                        panel.cursor = Some((cursor + 1).min(entries.len() - 1));
                    }
                }
            });

            ui.input(|x| {
                if x.key_pressed(egui::Key::Enter) {
                    if let Some(cursor) = panel.cursor {
                        if let Some(action) = entries.get(cursor) {
                            commands.entity(entity).despawn();
                            run.send(RunActionEvent(action.0.clone()));
                        }
                    }
                }
            });
        });

    if response.is_some_and(|x| x.response.clicked_elsewhere()) {
        commands.entity(entity).despawn();
    }
}
