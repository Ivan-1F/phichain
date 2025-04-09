use crate::action::{ActionRegistrationExt, ActionRegistry, RunActionEvent};
use crate::hotkey::modifier::Modifier;
use crate::hotkey::{Hotkey, HotkeyRegistry};
use crate::identifier::Identifier;
use crate::tab::TabRegistry;
use crate::UiState;
use bevy::app::App;
use bevy::prelude::{
    Commands, Component, Entity, EventWriter, IntoSystemConfigs, KeyCode, Plugin, Query, Res,
    ResMut, Update, Window, With, World,
};
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContext;
use egui::{TextEdit, Widget};
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

#[derive(Debug, Clone)]
enum ActionPanelEntryKind {
    Action,
    Tab,
}

#[derive(Debug, Clone)]
struct ActionPanelEntry {
    kind: ActionPanelEntryKind,
    id: Identifier,
    title: String,
    hotkey: Option<Hotkey>,
}

fn action_panel_ui_system(
    mut commands: Commands,
    actions: Res<ActionRegistry>,
    hotkeys: Res<HotkeyRegistry>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut context: Query<&mut EguiContext>,
    mut query: Query<(Entity, &mut ActionPanel)>,

    tab_registry: Res<TabRegistry>,
    mut ui_state: ResMut<UiState>,

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

    let mut entries = vec![];

    for (id, _) in actions
        .0
        .iter()
        .filter(|(id, _)| id.to_string() != "phichain.open_action_panel")
    {
        entries.push(ActionPanelEntry {
            kind: ActionPanelEntryKind::Action,
            id: id.clone(),
            title: t!(format!("action.{}", id).as_str()).to_string(),
            hotkey: hotkeys.0.get(id).cloned(),
        })
    }

    for (id, _) in tab_registry.iter() {
        entries.push(ActionPanelEntry {
            kind: ActionPanelEntryKind::Tab,
            id: id.clone(),
            title: t!(format!("tab.{}.title", id).as_str()).to_string(),
            hotkey: None,
        })
    }

    let response = egui::Modal::new("Action Panel".into())
        // .title_bar(false)
        // .collapsible(false)
        // .resizable(false)
        // .fixed_size((window.width() * 0.5, window.height() * 0.5))
        // .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.set_width(window.width() * 0.5);
            ui.set_height(window.height() * 0.5);
            ui.style_mut().interaction.selectable_labels = false;

            TextEdit::singleline(&mut panel.query)
                .desired_width(f32::INFINITY)
                .ui(ui)
                .request_focus();

            ui.separator();

            let entries = entries
                .iter()
                .filter(|entry| {
                    entry
                        .title
                        .to_ascii_lowercase()
                        .contains(panel.query.to_ascii_lowercase().as_str())
                        || entry
                            .id
                            .to_string()
                            .to_ascii_lowercase()
                            .contains(panel.query.to_ascii_lowercase().as_str())
                })
                .cloned()
                .collect::<Vec<_>>();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (index, entry) in entries.iter().enumerate() {
                    let selected = panel.cursor.is_some_and(|x| x == index);

                    egui::Frame::none()
                        .fill(if selected {
                            egui::Color32::from_rgba_unmultiplied(64, 94, 168, 100)
                        } else {
                            egui::Color32::TRANSPARENT
                        })
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let icon = match entry.kind {
                                    ActionPanelEntryKind::Action => egui_phosphor::regular::COMMAND,
                                    ActionPanelEntryKind::Tab => egui_phosphor::regular::BROWSER,
                                };
                                ui.label(format!("{} {}", icon, entry.title));

                                let remain = ui.available_width();
                                ui.add_space(remain - 200.0);

                                if let Some(hotkey) = &entry.hotkey {
                                    ui.label(hotkey.to_string());
                                }

                                ui.add_space(ui.available_width());
                            });
                        });
                }
            });

            ui.style_mut().interaction.selectable_labels = true;

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
                        if let Some(entry) = entries.get(cursor) {
                            commands.entity(entity).despawn();
                            match entry.kind {
                                ActionPanelEntryKind::Action => {
                                    run.send(RunActionEvent(entry.id.clone()));
                                }
                                ActionPanelEntryKind::Tab => {
                                    let id = entry.id.clone();

                                    if let Some(node) = ui_state.state.find_tab(&id) {
                                        ui_state.state.set_active_tab(node);
                                    } else {
                                        ui_state.state.add_window(vec![id]);
                                    }
                                }
                            }
                        }
                    }
                }
            });
        });

    if response.should_close() {
        commands.entity(entity).despawn();
    }
}
