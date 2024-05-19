use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use std::ops::Not;
use crate::action::ActionRegistry;

pub struct HotkeyPlugin;

impl Plugin for HotkeyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PressedKeys::default())
            .add_systems(
                Update,
                (listen_to_key_events_system, handle_hotkey_system).chain(),
            )
            .register_hotkey(vec![KeyCode::ControlLeft, KeyCode::KeyS]);
    }
}

trait HotkeyRegistrationExt {
    fn register_hotkey(&mut self, keys: impl IntoIterator<Item = KeyCode>) -> &mut Self;
}

impl HotkeyRegistrationExt for App {
    fn register_hotkey(&mut self, _keys: impl IntoIterator<Item = KeyCode>) -> &mut Self {
        self
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

fn handle_hotkey_system(
    world: &mut World
) {
    world.resource_scope(|world, keyboard: Mut<ButtonInput<KeyCode>>| {
       world.resource_scope(|world, mut registry: Mut<ActionRegistry>| {
           if keyboard.just_pressed(KeyCode::KeyD) {
               registry.run_action(world, "phichain.debug");
           }
       });
    });
}
