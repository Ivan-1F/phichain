use crate::action::ActionRegistrationExt;
use crate::hotkey::modifier::Modifier;
use crate::hotkey::Hotkey;
use crate::misc::WorkingDirectory;
use crate::notification::{ToastsExt, ToastsStorage};
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

pub struct ScreenshotPlugin;

impl Plugin for ScreenshotPlugin {
    fn build(&self, app: &mut App) {
        app.add_action(
            "phichain.take_screenshot",
            take_screenshot_system,
            Some(Hotkey::new(KeyCode::KeyP, vec![Modifier::Control])),
        );
    }
}

// FIXME: this is not working in Bevy 0.15, see https://github.com/bevyengine/bevy/issues/16689
fn take_screenshot_system(
    mut commands: Commands,
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
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path.clone()));
            toasts.success(t!(
                "screenshot.save.succeed",
                path = path.display().to_string()
            ));
        }
        Err(error) => {
            toasts.error(t!("screenshot.save.locate_failed", eror = error));
        }
    }
}
