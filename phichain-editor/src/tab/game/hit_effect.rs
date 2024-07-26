use crate::assets::ImageAssets;
use crate::project::project_loaded;
use crate::tab::game::GameViewport;
use crate::timing::{ChartTime, Paused};
use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy_hanabi::prelude::*;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::note::{Note, NoteKind};
use std::time::Duration;

const HOLD_PARTICLE_INTERVAL: f32 = 0.15;

pub struct HitEffectPlugin;

impl Plugin for HitEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_system).add_systems(
            Update,
            (
                spawn_hit_effect_system,
                update_hit_effect_system.after(TransformSystem::TransformPropagate),
                update_hit_effect_scale_system,
                animate_hit_effect_system,
            )
                .chain()
                .run_if(project_loaded()),
        );
    }
}

#[derive(Component, Debug)]
struct HitEffect(Vec2);

#[derive(Resource, Debug)]
struct TextureAtlasLayoutHandle(Handle<TextureAtlasLayout>);

#[derive(Resource, Debug)]
struct EffectAssetHandle(Handle<EffectAsset>);

fn setup_system(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = TextureAtlasLayout::from_grid(Vec2::splat(256.0), 1, 30, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    commands.insert_resource(TextureAtlasLayoutHandle(texture_atlas_layout.clone()));

    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(254.0 / 255.0, 1.0, 169.0 / 255.0, 1.0));
    gradient.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0));

    let writer = ExprWriter::new();
    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.).expr());
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(0.5).expr());
    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(40).expr(),
        dimension: ShapeDimension::Volume,
    };
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(100.).expr(),
    };

    let update_accel = AccelModifier::new(writer.lit(-6.0).expr());

    let effect = effects.add(
        EffectAsset::new(vec![4], Spawner::once(4.0.into(), true), writer.finish())
            .with_name("hit")
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .update(update_accel)
            .render(SetSizeModifier {
                size: CpuValue::Uniform((Vec2::new(7.0, 7.0), Vec2::new(10.0, 10.0))),
            })
            .render(ColorOverLifetimeModifier { gradient }),
    );

    commands.insert_resource(EffectAssetHandle(effect));

    // commands.spawn((
    //     Name::new("emit:once"),
    //     ParticleEffectBundle {
    //         effect: ParticleEffect::new(effect2),
    //         transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
    //         ..Default::default()
    //     },
    // ));
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_hit_effect_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut AnimationTimer, &mut TextureAtlas), With<HitEffect>>,
) {
    for (entity, mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            if atlas.index == 29 {
                commands.entity(entity).despawn();
            } else {
                atlas.index += 1;
            }
        }
    }
}

fn update_hit_effect_system(mut query: Query<(&mut Transform, &HitEffect)>) {
    for (mut transform, effect) in &mut query {
        transform.translation = Vec3::new(effect.0.x, effect.0.y, 10.0);
    }
}

fn update_hit_effect_scale_system(
    mut query: Query<&mut Transform, With<HitEffect>>,
    game_viewport: Res<GameViewport>,
) {
    for mut transform in &mut query {
        transform.scale = Vec3::splat(game_viewport.0.width() / 8000.0 * 6.0)
    }
}

#[derive(Component, Debug)]
struct PlayedHitEffect(f32);

fn spawn_hit_effect_system(
    mut commands: Commands,
    query: Query<(&Note, &GlobalTransform, Entity, Option<&PlayedHitEffect>)>,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
    assets: Res<ImageAssets>,
    paused: Res<Paused>,

    texture_atlas_layout_handle: Res<TextureAtlasLayoutHandle>,
    effect_asset_handle: Res<EffectAssetHandle>,
) {
    for (note, global_transform, entity, played) in &query {
        let mut spawn = || {
            let translation = global_transform.translation();
            commands.spawn((
                SpriteBundle {
                    texture: assets.hit.clone(),
                    sprite: Sprite {
                        color: Color::hex("#feffa9").unwrap(),
                        ..default()
                    },
                    ..default()
                },
                TextureAtlas {
                    layout: texture_atlas_layout_handle.0.clone(),
                    index: 0,
                },
                HitEffect(Vec2::new(translation.x, translation.y)),
                AnimationTimer(Timer::new(
                    Duration::from_millis(500 / 30),
                    TimerMode::Repeating,
                )),
            ));

            commands.spawn((
                ParticleEffectBundle {
                    effect: ParticleEffect::new(effect_asset_handle.0.clone()),
                    transform: Transform::from_translation(global_transform.translation()),
                    ..Default::default()
                },
                HitEffect(Vec2::new(translation.x, translation.y)),
            ));

            commands.entity(entity).insert(PlayedHitEffect(time.0));
        };

        let note_time = bpm_list.time_at(note.beat);

        match note.kind {
            NoteKind::Hold { .. } => {
                let end_time = bpm_list.time_at(note.end_beat());
                if note_time <= time.0
                    && time.0 <= end_time
                    && !paused.0
                    && (played.is_none()
                        || played.is_some_and(|last| (time.0 - last.0) > HOLD_PARTICLE_INTERVAL))
                {
                    spawn();
                }
            }
            _ => {
                if note_time <= time.0 && time.0 - note_time < 0.05 && played.is_none() && !paused.0
                {
                    spawn();
                }
            }
        }

        if note_time > time.0 && played.is_some() {
            commands.entity(entity).remove::<PlayedHitEffect>();
        }
    }
}
