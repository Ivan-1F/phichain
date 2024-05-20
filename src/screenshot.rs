use crate::action::ActionRegistrationExt;
use crate::hotkey::HotkeyRegistrationExt;
use crate::misc::WorkingDirectory;
use crate::notification::{ToastsExt, ToastsStorage};
use bevy::prelude::*;
use bevy::render::view::screenshot::ScreenshotManager;
use bevy::window::PrimaryWindow;

pub struct ScreenshotPlugin;

impl Plugin for ScreenshotPlugin {
    fn build(&self, app: &mut App) {
        app.register_action("phichain.take_screenshot", take_screenshot_system)
            .register_hotkey(
                "phichain.take_screenshot",
                vec![KeyCode::ControlLeft, KeyCode::KeyP],
            );
    }
}

fn take_screenshot_system(
    main_window: Query<Entity, With<PrimaryWindow>>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
    mut toasts: ResMut<ToastsStorage>,
    working_directory: Res<WorkingDirectory>,
) {
    match working_directory.screenshot() {
        Ok(screenshot_dir) => {
            let path = screenshot_dir.join(format!(
                "screenshot-{}.png",
                // `Local` conflicts with bevy::prelude::*, so use absolute path here
                chrono::prelude::Local::now().format("%Y-%m-%d-%H:%M:%S")
            ));
            match screenshot_manager.save_screenshot_to_disk(main_window.single(), path.clone()) {
                Ok(_) => {
                    toasts.success(format!("Screenshot saved to {:?}", path));
                }
                Err(_) => {
                    toasts.success("Failed to take screenshot");
                }
            };
        }
        Err(error) => {
            toasts.error(format!("Failed to locate screenshot directory: {}", error));
        }
    }
}
