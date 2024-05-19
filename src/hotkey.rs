use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use std::ops::Not;

pub struct HotkeyPlugin;

impl Plugin for HotkeyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PressedKeys::default())
            .add_systems(
                Update,
                (listen_to_key_events_system, handle_keybindings_system).chain(),
            )
            .register_hotkey(vec![KeyCode::ControlLeft, KeyCode::KeyS]);
    }
}

trait HotkeyRegistrationExt {
    fn register_hotkey(&mut self, keys: impl IntoIterator<Item = KeyCode>);
}

impl HotkeyRegistrationExt for App {
    fn register_hotkey(&mut self, _keys: impl IntoIterator<Item = KeyCode>) {
        unimplemented!()
    }
}

#[derive(Resource, Default, Debug, Clone)]
pub struct PressedKeys(pub Vec<KeyCode>);

fn listen_to_key_events_system(
    mut events: EventReader<KeyboardInput>,
    mut pressed_keys: ResMut<PressedKeys>,
) {
    for event in events.read() {
        if event.state.is_pressed() {
            pressed_keys
                .0
                .contains(&event.key_code)
                .not()
                .then(|| pressed_keys.0.push(event.key_code));
        } else {
            pressed_keys
                .0
                .contains(&event.key_code)
                .then(|| pressed_keys.0.retain(|x| x != &event.key_code));
        }
    }
}

fn handle_keybindings_system(
    mut events: EventReader<KeyboardInput>,
    pressed_keys: Res<PressedKeys>,
) {
    for event in events.read() {
        if event.state.is_pressed() && pressed_keys.0 == vec![KeyCode::SuperLeft, KeyCode::KeyS] {
            println!("save!");
        }
    }
}
