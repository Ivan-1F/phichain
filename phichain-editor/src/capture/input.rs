//! Input of the story in place of the real devices

use super::runner::Runner;
use bevy::ecs::system::SystemParam;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::prelude::*;
use bevy::window::{CursorEntered, CursorLeft, CursorMoved, Ime, PrimaryWindow, WindowEvent};

/// Writes input the way winit does: each event both as its own message and as a [`WindowEvent`]
#[derive(SystemParam)]
pub struct Inject<'w> {
    mouse: MessageWriter<'w, MouseButtonInput>,
    cursor: MessageWriter<'w, CursorMoved>,
    left: MessageWriter<'w, CursorLeft>,
    window: MessageWriter<'w, WindowEvent>,
}

impl Inject<'_> {
    pub fn mouse(&mut self, message: MouseButtonInput) {
        self.window.write(WindowEvent::MouseButtonInput(message));
        self.mouse.write(message);
    }

    pub fn cursor(&mut self, message: CursorMoved) {
        self.window.write(WindowEvent::CursorMoved(message.clone()));
        self.cursor.write(message);
    }

    pub fn left(&mut self, message: CursorLeft) {
        self.window.write(WindowEvent::CursorLeft(message.clone()));
        self.left.write(message);
    }
}

/// Drops what the real devices sent since the last frame, before the story writes its own input
#[allow(clippy::too_many_arguments)]
pub fn discard_real_input_system(
    mut keyboard: ResMut<Messages<KeyboardInput>>,
    mut ime: ResMut<Messages<Ime>>,
    mut mouse: ResMut<Messages<MouseButtonInput>>,
    mut wheel: ResMut<Messages<MouseWheel>>,
    mut cursor: ResMut<Messages<CursorMoved>>,
    mut entered: ResMut<Messages<CursorEntered>>,
    mut left: ResMut<Messages<CursorLeft>>,
    // only read by picking, which looks at the input it carries
    mut window: ResMut<Messages<WindowEvent>>,
) {
    keyboard.clear();
    ime.clear();
    mouse.clear();
    wheel.clear();
    cursor.clear();
    entered.clear();
    left.clear();
    window.clear();
}

/// Shows the story's cursor to systems reading [`Window::cursor_position`] during the frame
///
/// Overrides what winit recorded from the real cursor.
pub fn place_cursor_system(
    runner: Res<Runner>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = windows.single_mut() {
        window.set_cursor_position(runner.cursor());
    }
}

/// Clears the cursor before winit syncs window changes in `Last`, which would otherwise warp the real cursor
pub fn lift_cursor_system(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = windows.single_mut() {
        if window.cursor_position().is_some() {
            window.set_cursor_position(None);
        }
    }
}
