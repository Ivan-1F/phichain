use bevy::prelude::*;
use bevy_egui::EguiContexts;

/// When egui wants keyboard inputs (editing [`egui::TextEdit`] or [`egui::DragValue`], etc.), we mute them for bevy
///
/// This fixed keyboard events being triggered when typing in egui inputs
pub fn mute_keyboard_for_bevy_when_egui_wants_system(
    mut contexts: EguiContexts,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
) {
    let ctx = contexts.ctx_mut();
    if ctx.wants_keyboard_input() {
        let modifiers = [
            KeyCode::SuperLeft,
            KeyCode::SuperRight,
            KeyCode::ControlLeft,
            KeyCode::ControlRight,
            KeyCode::AltLeft,
            KeyCode::AltRight,
            KeyCode::ShiftLeft,
            KeyCode::ShiftRight,
        ];

        let pressed = modifiers.map(|key| keyboard.pressed(key).then_some(key));

        keyboard.reset_all();

        for key in pressed.into_iter().flatten() {
            keyboard.press(key);
        }
    }
}
