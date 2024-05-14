use bevy::prelude::*;

use crate::{constants::ILLUSTRATION_ALPHA, project::project_loaded};

use super::GameViewport;

pub struct IllustrationPlugin;

impl Plugin for IllustrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnIllustrationEvent>()
            .add_systems(Update, resize_illustration_system.run_if(project_loaded()))
            .add_systems(Update, spawn_illustration_system);
    }
}

#[derive(Component)]
pub struct Illustration;

#[derive(Event)]
pub struct SpawnIllustrationEvent(pub Handle<Image>);

fn spawn_illustration_system(
    mut commands: Commands,
    mut events: EventReader<SpawnIllustrationEvent>,
) {
    if events.len() > 1 {
        warn!("Mutiple illustration are requested, ignoring previous ones");
    }

    // TODO: check if illustration already exists
    if let Some(event) = events.read().last() {
        commands.spawn((
            SpriteBundle {
                texture: event.0.clone(),
                sprite: Sprite {
                    // TODO: make this update in a Update system
                    color: Color::WHITE.with_a(ILLUSTRATION_ALPHA),
                    ..default()
                },
                ..default()
            },
            Illustration,
        ));
    }
}

fn resize_illustration_system(
    mut query: Query<&mut Sprite, With<Illustration>>,
    viewport: Res<GameViewport>,
) {
    let mut illustration = query.single_mut();

    illustration.custom_size = Some(viewport.0.size());
}
