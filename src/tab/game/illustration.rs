use bevy::prelude::*;

use crate::constants::{ILLUSTRATION_ALPHA, ILLUSTRATION_BLUR};

use super::GameViewport;

pub struct IllustrationPlugin;

impl Plugin for IllustrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_illustration_system)
            .add_systems(Update, resize_illustration_system)
            .add_systems(Update, blur_image_system);
    }
}

#[derive(Component)]
struct Illustration;

#[derive(Resource)]
struct IllustrationHandle(Handle<Image>);

/// Blur illustration
fn blur_image_system(
    mut events: EventReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
    illustration_handle: Res<IllustrationHandle>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Added { id } => {
                if *id == illustration_handle.0.id() {
                    if let Some(illustration) = images.get_mut(*id) {
                        let width = illustration.width();
                        let height = illustration.height();

                        // Extract RGB components, skipping alpha values
                        let mut rgb_data: Vec<[u8; 3]> = illustration
                            .data
                            .chunks_exact(4)
                            .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                            .collect();

                        // Apply Gaussian blur on the RGB data
                        fastblur::gaussian_blur(&mut rgb_data, width as usize, height as usize, ILLUSTRATION_BLUR);

                        // Recombine RGB data with Alpha channel
                        let mut recombined_data = Vec::with_capacity(illustration.data.len());
                        let mut alpha_iter = illustration.data.chunks_exact(4).map(|chunk| chunk[3]);
                        for rgb in rgb_data.iter() {
                            recombined_data.extend_from_slice(rgb);
                            recombined_data.push(alpha_iter.next().unwrap_or(255)); // Default to 255 if alpha channel is missing
                        }

                        // Update illustration data
                        illustration.data = recombined_data;
                    }
                }
            }
            _ => {}
        }
    }
}

fn spawn_illustration_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture: Handle<Image> = asset_server.load("image/illustration.png");
    commands.insert_resource(IllustrationHandle(texture.clone()));
    commands.spawn((
        SpriteBundle {
            texture,
            sprite: Sprite {
                color: Color::WHITE.with_a(ILLUSTRATION_ALPHA),
                ..default()
            },
            ..default()
        },
        Illustration,
    ));
}

fn resize_illustration_system(
    mut query: Query<&mut Sprite, With<Illustration>>,
    viewport: Res<GameViewport>,
) {
    let mut illustration = query.single_mut();

    illustration.custom_size = Some(viewport.0.size());
}
