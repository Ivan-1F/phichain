use std::path::PathBuf;

use bevy::{prelude::*, render::render_asset::RenderAssetUsages};

use crate::{
    constants::{ILLUSTRATION_ALPHA, ILLUSTRATION_BLUR},
    GameSet,
};

use super::GameViewport;

pub struct IllustrationPlugin;

impl Plugin for IllustrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                resize_illustration_system,
                update_alpha_system,
                place_everything_above_illustration_system,
            )
                .in_set(GameSet)
                .run_if(any_with_component::<Illustration>),
        );
    }
}

#[derive(Resource)]
pub struct IllustrationAssetId(pub AssetId<Image>);

#[derive(Component)]
pub struct Illustration;

pub fn load_illustration(path: PathBuf, commands: &mut Commands) {
    let image = image::open(path).unwrap().blur(ILLUSTRATION_BLUR);
    let is_srgb = matches!(
        image.color(),
        image::ColorType::Rgb8 | image::ColorType::Rgba8
    );

    commands.queue(move |world: &mut World| {
        world.resource_scope(|world, mut images: Mut<Assets<Image>>| {
            if world.query::<&Illustration>().get_single(world).is_ok() {
                warn!("Trying to spawn illustration with Illustration exists");
                return;
            }
            let handle = images.add(Image::from_dynamic(
                image,
                is_srgb,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            ));
            world.insert_resource(IllustrationAssetId(handle.id()));
            world.spawn((Sprite::from_image(handle), Illustration));
        });
    });
}

fn update_alpha_system(mut query: Query<&mut Sprite, With<Illustration>>) {
    let mut illustration = query.single_mut();
    illustration.color.set_alpha(ILLUSTRATION_ALPHA);
}

fn resize_illustration_system(
    mut query: Query<&mut Sprite, With<Illustration>>,
    viewport: Res<GameViewport>,
) {
    let mut illustration = query.single_mut();

    illustration.custom_size = Some(viewport.0.size());
}

fn place_everything_above_illustration_system(
    mut query: Query<&mut Transform, Without<Illustration>>,
) {
    for mut transform in &mut query {
        transform.translation.z = 1.0;
    }
}
