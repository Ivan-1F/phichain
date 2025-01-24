use crate::action::{ActionRegistrationExt, ActionRegistry};
use crate::hotkey::modifier::Modifier;
use crate::hotkey::{Hotkey, HotkeyRegistry};
use bevy::app::App;
use bevy::prelude::{
    Commands, Component, Entity, IntoSystemConfigs, KeyCode, Plugin, Query, Res, Update, Window,
    With, World,
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
pub struct ActionPanel(String);

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

    egui::Window::new("Action Panel")
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .fixed_size((window.width() * 0.6, window.height() * 0.5))
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.style_mut().interaction.selectable_labels = false;

            ui.heading("Action Panel");
            ui.separator();
            TextEdit::singleline(&mut panel.0)
                .desired_width(f32::INFINITY)
                .ui(ui)
                .request_focus();

            ui.separator();

            for (id, action) in &actions.0 {
                ui.horizontal(|ui| {
                    ui.label(id.to_string());

                    let remain = ui.available_width();
                    ui.add_space(remain - 200.0);

                    if action.enable_hotkey {
                        if let Some(hotkey) = hotkeys.0.get(id) {
                            ui.label(hotkey.to_string());
                        }
                    }
                });
            }

            ui.style_mut().interaction.selectable_labels = true;

            ui.input(|x| {
                if x.key_pressed(egui::Key::Escape) {
                    commands.entity(entity).despawn();
                }
            });
        });
}
