//! Input of the story in place of the real devices

use super::runner::Runner;
use bevy::ecs::system::SystemParam;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKeyCode};
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::window::{CursorEntered, CursorLeft, CursorMoved, Ime, PrimaryWindow, WindowEvent};
use bevy_egui::{egui, EguiContexts};

/// Writes input the way winit does: each event both as its own message and as a [`WindowEvent`]
#[derive(SystemParam)]
pub struct Inject<'w> {
    keyboard: MessageWriter<'w, KeyboardInput>,
    mouse: MessageWriter<'w, MouseButtonInput>,
    cursor: MessageWriter<'w, CursorMoved>,
    left: MessageWriter<'w, CursorLeft>,
    window: MessageWriter<'w, WindowEvent>,
}

impl Inject<'_> {
    pub fn key(&mut self, message: KeyboardInput) {
        self.window
            .write(WindowEvent::KeyboardInput(message.clone()));
        self.keyboard.write(message);
    }

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

/// A key of the story as winit reports it
pub fn key_message(code: KeyCode, state: ButtonState, window: Entity) -> KeyboardInput {
    KeyboardInput {
        key_code: code,
        logical_key: logical_key(code),
        state,
        text: None,
        repeat: false,
        window,
    }
}

/// The name the documentation formats into the reader's key labels, such as `Space` or `R`
pub fn key_name(code: KeyCode) -> String {
    match logical_key(code) {
        Key::Character(text) => text.to_uppercase(),
        key => format!("{key:?}"),
    }
}

fn logical_key(code: KeyCode) -> Key {
    let character = |text: &str| Key::Character(text.into());
    match code {
        KeyCode::Space => Key::Space,
        KeyCode::Enter => Key::Enter,
        KeyCode::Escape => Key::Escape,
        KeyCode::BracketLeft => character("["),
        KeyCode::BracketRight => character("]"),
        code => {
            // letters are the remaining keys stories press
            let name = format!("{code:?}");
            let letter = name
                .strip_prefix("Key")
                .filter(|letter| letter.len() == 1)
                .unwrap_or_else(|| panic!("stories cannot press {code:?} yet"));
            character(&letter.to_lowercase())
        }
    }
}

/// Typing one character into whatever has keyboard focus
pub fn text_message(character: char, state: ButtonState, window: Entity) -> KeyboardInput {
    KeyboardInput {
        key_code: KeyCode::Unidentified(NativeKeyCode::Unidentified),
        logical_key: Key::Character(character.to_string().into()),
        state,
        text: (state == ButtonState::Pressed).then(|| character.to_string().into()),
        repeat: false,
        window,
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

/// Draws the story's cursor, as screenshots do not include the system one
pub fn draw_cursor_system(mut contexts: EguiContexts, runner: Res<Runner>) -> Result {
    let Some(cursor) = runner.cursor() else {
        return Ok(());
    };
    let painter = contexts.ctx_mut()?.layer_painter(egui::LayerId::new(
        egui::Order::Debug,
        egui::Id::new("capture_cursor"),
    ));
    let origin = egui::pos2(cursor.x, cursor.y);
    let point = |x: f32, y: f32| origin + egui::vec2(x, y) * 1.5;
    // an arrow, filled as two convex parts and outlined as a whole
    let head = vec![
        point(0.0, 0.0),
        point(0.0, 16.0),
        point(4.2, 12.2),
        point(6.9, 11.2),
        point(12.0, 11.2),
    ];
    let tail = vec![
        point(4.2, 12.2),
        point(7.0, 18.5),
        point(9.6, 17.4),
        point(6.9, 11.2),
    ];
    let outline = vec![
        point(0.0, 0.0),
        point(0.0, 16.0),
        point(4.2, 12.2),
        point(7.0, 18.5),
        point(9.6, 17.4),
        point(6.9, 11.2),
        point(12.0, 11.2),
    ];
    painter.add(egui::Shape::convex_polygon(
        head,
        egui::Color32::BLACK,
        egui::Stroke::NONE,
    ));
    painter.add(egui::Shape::convex_polygon(
        tail,
        egui::Color32::BLACK,
        egui::Stroke::NONE,
    ));
    painter.add(egui::Shape::closed_line(
        outline,
        egui::Stroke::new(1.5_f32, egui::Color32::WHITE),
    ));
    Ok(())
}
