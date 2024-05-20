use std::path::PathBuf;

use bevy::{prelude::*, render::render_asset::RenderAssetUsages};

use crate::notification::{ToastsExt, ToastsStorage};
use crate::{
    constants::{ILLUSTRATION_ALPHA, ILLUSTRATION_BLUR},
    project::project_loaded,
};

use super::GameViewport;

pub struct IllustrationPlugin;

impl Plugin for IllustrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnIllustrationEvent>()
            .add_systems(
                Update,
                (resize_illustration_system, update_alpha_system)
                    .run_if(project_loaded().and_then(any_with_component::<Illustration>)),
            )
            .add_systems(Update, spawn_illustration_system);
    }
}

#[derive(Component)]
pub struct Illustration;

#[derive(Event)]
pub struct SpawnIllustrationEvent(pub PathBuf);

fn spawn_illustration_system(
    mut commands: Commands,
    mut events: EventReader<SpawnIllustrationEvent>,
    mut images: ResMut<Assets<Image>>,
    query: Query<&Illustration>,
    mut toasts_storage: ResMut<ToastsStorage>,
) {
    if events.len() > 1 {
        warn!("Multiple illustrations are requested, ignoring previous ones");
    }

    if let Some(event) = events.read().last() {
        if query.get_single().is_ok() {
            warn!("Trying to spawn illustration with Illustration exists");
            return;
        }

        match image::open(event.0.clone()) {
            Ok(image) => {
                let image = image.blur(ILLUSTRATION_BLUR);
                let is_srgb = matches!(
                    image.color(),
                    image::ColorType::Rgb8 | image::ColorType::Rgba8
                );
                let handle = images.add(Image::from_dynamic(
                    image,
                    is_srgb,
                    RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
                ));
                commands.spawn((
                    SpriteBundle {
                        texture: handle,
                        ..default()
                    },
                    Illustration,
                ));
            }
            Err(error) => {
                toasts_storage.error(t!("illustration.load.failed", error = error.to_string()))
            }
        }
    }
}

fn update_alpha_system(mut query: Query<&mut Sprite, With<Illustration>>) {
    let mut illustration = query.single_mut();
    illustration.color.set_a(ILLUSTRATION_ALPHA);
}

fn resize_illustration_system(
    mut query: Query<&mut Sprite, With<Illustration>>,
    viewport: Res<GameViewport>,
) {
    let mut illustration = query.single_mut();

    illustration.custom_size = Some(viewport.0.size());
}
