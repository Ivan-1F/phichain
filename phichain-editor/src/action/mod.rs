use crate::identifier::Identifier;
use bevy::{prelude::*, utils::HashMap};

pub type ActionIdentifier = Identifier;

pub struct RegisteredAction {
    system: Box<dyn System<In = (), Out = ()>>,
}

impl RegisteredAction {
    pub fn run(&mut self, world: &mut World) {
        self.system.run((), world);
    }
}

#[derive(Resource, Deref, Default)]
pub struct ActionRegistry(HashMap<ActionIdentifier, RegisteredAction>);

impl ActionRegistry {
    pub fn run_action(&mut self, world: &mut World, id: impl Into<ActionIdentifier>) {
        let id = id.into();
        if let Some(action) = self.0.get_mut(&id) {
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
            .register_action("phichain.debug", || {
                println!("Hello from Phichain!");
            });
    }
}

pub trait ActionRegistrationExt {
    fn register_action<M1>(
        &mut self,
        id: impl Into<ActionIdentifier>,
        system: impl IntoSystem<(), (), M1>,
    ) -> &mut Self;
}

impl ActionRegistrationExt for App {
    fn register_action<M1>(
        &mut self,
        id: impl Into<ActionIdentifier>,
        system: impl IntoSystem<(), (), M1>,
    ) -> &mut Self {
        self.world
            .resource_scope(|world, mut registry: Mut<ActionRegistry>| {
                registry.0.insert(
                    id.into(),
                    RegisteredAction {
                        system: Box::new({
                            let mut sys = IntoSystem::into_system(system);
                            sys.initialize(world);
                            sys
                        }),
                    },
                )
            });
        self
    }
}
