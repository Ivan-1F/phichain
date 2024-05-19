use crate::action::ActionRegistry;
use crate::identifier::Identifier;
use bevy::ecs::system::SystemState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::utils::HashMap;
use std::ops::Not;

pub struct HotkeyPlugin;

pub type HotkeyIdentifier = Identifier;

impl Plugin for HotkeyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PressedKeys>()
            .init_resource::<HotkeyRegistry>()
            .add_systems(
                Update,
                (listen_to_key_events_system, handle_hotkey_system).chain(),
            )
            .register_hotkey("phichain.debug", vec![KeyCode::ControlLeft, KeyCode::KeyD]);
    }
}

#[derive(Resource, Debug, Default)]
struct HotkeyRegistry(HashMap<HotkeyIdentifier, Vec<KeyCode>>);

trait HotkeyRegistrationExt {
    fn register_hotkey(
        &mut self,
        id: impl Into<HotkeyIdentifier>,
        keys: impl IntoIterator<Item = KeyCode>,
    ) -> &mut Self;
}

impl HotkeyRegistrationExt for App {
    fn register_hotkey(
        &mut self,
        id: impl Into<HotkeyIdentifier>,
        keys: impl IntoIterator<Item = KeyCode>,
    ) -> &mut Self {
        self.world
            .resource_mut::<HotkeyRegistry>()
            .0
            .insert(id.into(), IntoIterator::into_iter(keys).collect());

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
    world: &mut World,
    state: &mut SystemState<(
        EventReader<KeyboardInput>,
        Res<PressedKeys>,
        Res<HotkeyRegistry>,
    )>,
) {
    let (mut events, pressed_keys, hotkey) = state.get(world);

    let mut actions_to_run = vec![];

    for event in events.read() {
        if event.state.is_pressed() {
            for (id, keys) in hotkey.0.clone() {
                if pressed_keys.0 == keys {
                    actions_to_run.push(id);
                }
            }
        }
    }

    world.resource_scope(|world, mut registry: Mut<ActionRegistry>| {
        for action in actions_to_run {
            registry.run_action(world, action);
        }
    });
}
