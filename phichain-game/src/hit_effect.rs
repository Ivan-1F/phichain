use crate::constants::PERFECT_COLOR;
use crate::layer::HIT_EFFECT_LAYER;
use crate::scale::NoteScale;
use crate::{ChartTime, GameConfig, GameSet, GameViewport, Paused};
use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy_prototype_lyon::prelude::{Fill, GeometryBuilder, ShapeBundle};
use bevy_prototype_lyon::shapes;
use phichain_assets::ImageAssets;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::easing::Easing;
use phichain_chart::note::{Note, NoteKind};
use rand::Rng;
use std::time::Duration;

const HOLD_PARTICLE_INTERVAL: f32 = 0.15;
const HIT_EFFECT_DURATION: Duration = Duration::from_millis(500);
const HIT_EFFECT_FRAMES: u32 = 30;

pub struct HitEffectPlugin;

/// A simple timer for hit effects, compat layer for [`GameConfig::hit_effect_follow_game_time`]
#[derive(Debug, Clone, Default, Resource)]
pub struct HitEffectTime {
    current: f32,
    delta: f32,
}

impl HitEffectTime {
    pub fn delta_seconds(&self) -> f32 {
        self.delta
    }

    pub fn delta(&self) -> Duration {
        Duration::from_secs_f32(self.delta_seconds())
    }
}

fn update_hit_effect_time_system(
    mut het: ResMut<HitEffectTime>,
    game_config: Res<GameConfig>,
    chart_time: Res<ChartTime>,
    time: Res<Time>,
) {
    if game_config.hit_effect_follow_game_time {
        // based on game time
        het.delta = chart_time.0 - het.current;
        het.current = chart_time.0;
    } else {
        // based on bevy's built in timer
        het.delta = time.delta_secs();
    }
}

impl Plugin for HitEffectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HitEffectTime>()
            .add_systems(Startup, setup_system)
            .add_systems(
                Update,
                (
                    spawn_hit_effect_system.after(TransformSystem::TransformPropagate),
                    update_hit_effect_system,
                    update_hit_effect_scale_system,
                    animate_hit_effect_system,
                )
                    .chain()
                    .in_set(GameSet),
            )
            .add_systems(PreUpdate, update_hit_effect_time_system)
            .add_systems(
                Update,
                (
                    update_lifetime_system,
                    update_opacity_system,
                    update_velocity_system,
                    update_translation_system,
                    despawn_system,
                )
                    .chain()
                    .in_set(GameSet),
            );
    }
}

#[derive(Component, Debug)]
struct HitEffect(Vec2);

#[derive(Resource, Debug)]
struct TextureAtlasLayoutHandle(Handle<TextureAtlasLayout>);

fn setup_system(
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(256), 1, HIT_EFFECT_FRAMES, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    commands.insert_resource(TextureAtlasLayoutHandle(texture_atlas_layout.clone()));
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_hit_effect_system(
    mut commands: Commands,
    time: Res<HitEffectTime>,
    mut query: Query<(Entity, &mut AnimationTimer, &mut Sprite), With<HitEffect>>,
) {
    for (entity, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                if atlas.index == 29 {
                    commands.entity(entity).despawn();
                } else {
                    atlas.index += 1;
                }
            }
        }
    }
}

fn update_hit_effect_system(mut query: Query<(&mut Transform, &HitEffect)>) {
    for (mut transform, effect) in &mut query {
        transform.translation = Vec3::new(effect.0.x, effect.0.y, HIT_EFFECT_LAYER);
    }
}

fn update_hit_effect_scale_system(
    mut query: Query<&mut Transform, With<HitEffect>>,
    note_scale: Res<NoteScale>,
) {
    for mut transform in &mut query {
        transform.scale = Vec3::splat(note_scale.0 * 6.0)
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

    game_viewport: Res<GameViewport>,

    config: Res<GameConfig>,
) {
    if config.hide_hit_effect {
        return;
    }

    for (note, global_transform, entity, played) in &query {
        let mut spawn = || {
            let translation = global_transform.translation();

            let mut sprite = Sprite::from_atlas_image(
                assets.hit.clone(),
                TextureAtlas {
                    layout: texture_atlas_layout_handle.0.clone(),
                    index: 0,
                },
            );

            sprite.color = PERFECT_COLOR;

            commands.spawn((
                sprite,
                HitEffect(Vec2::new(translation.x, translation.y)),
                AnimationTimer(Timer::new(
                    HIT_EFFECT_DURATION / HIT_EFFECT_FRAMES,
                    TimerMode::Repeating,
                )),
            ));

            let factor = game_viewport.0.width() / 426.0;

            for _ in 0..4 {
                commands.spawn(HitParticleBundle::new(
                    global_transform.translation().truncate(),
                    factor,
                ));
            }

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

#[derive(Bundle)]
pub struct HitParticleBundle {
    hit_particle: HitParticle,
    velocity: Velocity,
    direction: Direction,
    lifetime: Lifetime,
    shape: ShapeBundle,
    fill: Fill,
}

impl HitParticleBundle {
    pub fn new(position: Vec2, factor: f32) -> Self {
        let size = rand::thread_rng().gen_range(7.0..=10.0) * factor;
        let shape = shapes::Rectangle {
            extents: Vec2::splat(size),
            origin: Default::default(),
            ..default()
        };

        let angle = rand::thread_rng().gen_range(-std::f32::consts::PI..=std::f32::consts::PI);
        let quat = Quat::from_rotation_z(angle);

        Self {
            hit_particle: Default::default(),
            velocity: Default::default(),
            direction: Direction(quat),
            lifetime: Default::default(),
            shape: ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                transform: Transform {
                    translation: position.extend(HIT_EFFECT_LAYER),
                    ..default()
                },
                ..default()
            },
            fill: Fill::color(PERFECT_COLOR),
        }
    }
}

#[derive(Debug, Component, Clone, Default)]
struct HitParticle;
#[derive(Debug, Component, Clone, Default)]
struct Velocity(Vec2);
#[derive(Debug, Component, Clone, Default)]
struct Direction(Quat);
#[derive(Debug, Component, Clone, Default)]
struct Lifetime(f32);

fn update_lifetime_system(
    mut query: Query<&mut Lifetime, With<HitParticle>>,
    time: Res<HitEffectTime>,
) {
    for mut lifetime in &mut query {
        lifetime.0 += time.delta_seconds();
    }
}

fn update_opacity_system(mut query: Query<(&mut Fill, &Lifetime), With<HitParticle>>) {
    for (mut fill, lifetime) in &mut query {
        fill.color.set_alpha((0.5 - lifetime.0) / 0.5);
    }
}

fn update_velocity_system(
    mut query: Query<(&mut Velocity, &Direction, &Lifetime), With<HitParticle>>,
    game_viewport: Res<GameViewport>,
) {
    for (mut velocity, direction, lifetime) in &mut query {
        velocity.0 = (direction.0 * Vec3::new(1.0, 1.0, 0.0) * 150.0).truncate()
            * Easing::EaseOutSine.ease((0.5 - lifetime.0) / 0.5)
            * game_viewport.0.width()
            / 426.0;
    }
}

fn update_translation_system(
    mut query: Query<(&mut Transform, &Velocity), With<HitParticle>>,
    time: Res<HitEffectTime>,
) {
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.0.extend(0.0) * time.delta_seconds();
    }
}

fn despawn_system(mut commands: Commands, query: Query<(Entity, &Lifetime), With<HitParticle>>) {
    for (entity, lifetime) in &query {
        if lifetime.0 >= 0.5 {
            commands.entity(entity).despawn();
        }
    }
}
