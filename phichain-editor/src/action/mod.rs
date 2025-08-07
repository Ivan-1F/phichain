use crate::hotkey::{Hotkey, HotkeyContext, HotkeyExt};
use crate::identifier::Identifier;
use crate::telemetry::PushTelemetryEvent;
use bevy::ecs::system::{BoxedSystem, SystemState};
use bevy::log;
use bevy::prelude::*;
use indexmap::IndexMap;
use phichain_game::GameSet;
use serde_json::json;

pub type ActionIdentifier = Identifier;

// TODO: hold action's name
pub struct RegisteredAction {
    system: BoxedSystem<(), Result>,
    pub enable_hotkey: bool,
    pub is_heavy: bool,
}

impl RegisteredAction {
    pub fn run(&mut self, world: &mut World) {
        match self.system.run((), world) {
            Ok(_) => {}
            Err(error) => {
                // TODO: show a toast here
                log::error!("Action failed: {}", error);
            }
        };
    }
}

#[derive(Resource, Deref, Default)]
pub struct ActionRegistry(pub IndexMap<ActionIdentifier, RegisteredAction>);

impl ActionRegistry {
    pub fn run_action(&mut self, world: &mut World, id: impl Into<ActionIdentifier>) {
        let id = id.into();
        if let Some(action) = self.0.get_mut(&id) {
            if action.is_heavy {
                world.send_event(PushTelemetryEvent::new(
                    "phichain.editor.action.invoked",
                    json!({ "action": id }),
                ));
            }

            action.run(world);
        } else {
            error!("Failed to find action with id {}", id);
        }
    }
}

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionRegistry>()
            .add_systems(Update, handle_action_hotkey_system.in_set(GameSet))
            .add_event::<RunActionEvent>()
            .add_observer(handle_run_action_event_system);
    }
}

fn add_action_impl<M1>(
    app: &mut App,
    id: impl Into<ActionIdentifier>,
    system: impl IntoSystem<(), Result, M1>,
    hotkey: Option<Hotkey>,
    heavy: bool,
) {
    let id = id.into();

    let action = RegisteredAction {
        system: Box::new({
            let mut sys = IntoSystem::into_system(system);
            sys.initialize(app.world_mut());
            sys
        }),
        enable_hotkey: hotkey.is_some(),
        is_heavy: heavy,
    };

    app.world_mut()
        .resource_mut::<ActionRegistry>()
        .0
        .insert(id.clone(), action);

    if let Some(hotkey) = hotkey {
        app.add_hotkey(id, hotkey);
    }
}

pub trait ActionRegistrationExt {
    fn add_action<M1>(
        &mut self,
        id: impl Into<ActionIdentifier>,
        system: impl IntoSystem<(), Result, M1>,
        hotkey: Option<Hotkey>,
    ) -> &mut Self;

    fn add_heavy_action<M1>(
        &mut self,
        id: impl Into<ActionIdentifier>,
        system: impl IntoSystem<(), Result, M1>,
        hotkey: Option<Hotkey>,
    ) -> &mut Self;
}

impl ActionRegistrationExt for App {
    fn add_action<M1>(
        &mut self,
        id: impl Into<ActionIdentifier>,
        system: impl IntoSystem<(), Result, M1>,
        hotkey: Option<Hotkey>,
    ) -> &mut Self {
        add_action_impl(self, id, system, hotkey, false);
        self
    }

    fn add_heavy_action<M1>(
        &mut self,
        id: impl Into<ActionIdentifier>,
        system: impl IntoSystem<(), Result, M1>,
        hotkey: Option<Hotkey>,
    ) -> &mut Self {
        add_action_impl(self, id, system, hotkey, true);
        self
    }
}

fn handle_action_hotkey_system(world: &mut World) {
    let mut state: SystemState<(HotkeyContext, Res<ActionRegistry>)> = SystemState::new(world);
    let (hotkey, registry) = state.get_mut(world);
    let mut actions_to_run = vec![];

    for (id, _) in registry.0.iter().filter(|(_, action)| action.enable_hotkey) {
        if hotkey.just_pressed(id.clone()) {
            actions_to_run.push(id.clone());
        }
    }

    if !actions_to_run.is_empty() {
        world.resource_scope(|world, mut registry: Mut<ActionRegistry>| {
            for action in actions_to_run {
                registry.run_action(world, action);
            }
        });
    }
}

#[derive(Debug, Clone, Event)]
pub struct RunActionEvent(pub Identifier);

fn handle_run_action_event_system(trigger: Trigger<RunActionEvent>, world: &mut World) {
    world.resource_scope(|world, mut registry: Mut<ActionRegistry>| {
        registry.run_action(world, trigger.event().0.clone());
    });
}
