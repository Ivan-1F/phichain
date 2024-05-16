use std::path::PathBuf;

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

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
) {
    if events.len() > 1 {
        warn!("Multiple illustrations are requested, ignoring previous ones");
    }

    if let Some(event) = events.read().last() {
        if query.get_single().is_ok() {
            warn!("Trying to spawn illustration with Illustration exists");
            return;
        }

        // TODO: error handling
        let image = image::open(event.0.clone())
            .unwrap()
            .blur(ILLUSTRATION_BLUR);
        let rgb8 = image.as_rgba8().unwrap();
        let handle = images.add(Image::new(
            Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            rgb8.clone().into_vec(),
            TextureFormat::Rgba8UnormSrgb,
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
