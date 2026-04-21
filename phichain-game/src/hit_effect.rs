use crate::constants::PERFECT_COLOR;
use crate::event::Events;
use crate::layer::HIT_EFFECT_LAYER;
use crate::{ChartTime, GameConfig, GameSet, GameViewport, Paused};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes;
use phichain_assets::{HitEffectAtlas, ImageAssets, RespackMeta};
use phichain_chart::bpm_list::BpmList;
use phichain_chart::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};
use phichain_chart::easing::Easing;
use phichain_chart::event::{EventEvaluationResult, LineEvent, LineEventKind};
use phichain_chart::note::{Note, NoteKind};
use rand::Rng;
use std::time::Duration;

/// Hit effect sprite width as a multiple of the reference note width.
/// Equivalent to the legacy formula `(256 * 6 / 8000) * viewport`
const HIT_FX_NOTE_WIDTH_RATIO: f32 = 256.0 * 6.0 / 989.0;

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
            .add_systems(
                Update,
                (
                    spawn_hit_effect_system,
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

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_hit_effect_system(
    mut commands: Commands,
    time: Res<HitEffectTime>,
    atlas_res: Res<HitEffectAtlas>,
    mut query: Query<(Entity, &mut AnimationTimer, &mut Sprite), With<HitEffect>>,
) {
    let last_frame = atlas_res.frame_count - 1;
    for (entity, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                if atlas.index == last_frame as usize {
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
    viewport: Res<GameViewport>,
    config: Res<GameConfig>,
    atlas: Res<HitEffectAtlas>,
    meta: Res<RespackMeta>,
) {
    let target_width = crate::scale::reference_note_width(viewport.0.width(), config.note_scale)
        * HIT_FX_NOTE_WIDTH_RATIO
        * meta.hit_fx.scale;
    let scale = target_width / atlas.frame_size.x as f32;
    for mut transform in &mut query {
        transform.scale = Vec3::splat(scale);
    }
}

#[derive(Component, Debug)]
struct PlayedHitEffect(f32);

/// Evaluate line events at a given beat and return (x, y, rotation) values.
fn evaluate_line_at_beat(
    beat: f32,
    events: &Events,
    line_event_query: &Query<&LineEvent>,
) -> (f32, f32, f32) {
    let mut x_value = EventEvaluationResult::Unaffected;
    let mut y_value = EventEvaluationResult::Unaffected;
    let mut rotation_value = EventEvaluationResult::Unaffected;

    for event in events.iter().filter_map(|e| line_event_query.get(e).ok()) {
        let value = event.evaluate_inclusive(beat);
        match event.kind {
            LineEventKind::X => x_value = x_value.max(value),
            LineEventKind::Y => y_value = y_value.max(value),
            LineEventKind::Rotation => rotation_value = rotation_value.max(value),
            _ => {}
        }
    }

    (
        x_value.value().unwrap_or(0.0),
        y_value.value().unwrap_or(0.0),
        rotation_value.value().unwrap_or(0.0),
    )
}

/// Compute the world position for a hit effect given line event values and note x offset.
fn compute_hit_effect_position(
    line_x: f32,
    line_y: f32,
    line_rotation_deg: f32,
    note_x: f32,
    game_viewport: &GameViewport,
) -> Vec2 {
    let vw = game_viewport.0.width();
    let vh = game_viewport.0.height();
    let line_scale = crate::scale::line_world_scale(vw);

    // line world position
    let world_line_x = line_x / CANVAS_WIDTH * vw;
    let world_line_y = line_y / CANVAS_HEIGHT * vh;

    // note x offset in world space (same formula as update_note_system, then multiplied by line scale)
    let local_note_x = (note_x / CANVAS_WIDTH) * vw / line_scale;
    let scaled_note_x = local_note_x * line_scale;

    // rotate note offset by line rotation
    let rotation_rad = line_rotation_deg.to_radians();
    let cos = rotation_rad.cos();
    let sin = rotation_rad.sin();

    Vec2::new(
        world_line_x + scaled_note_x * cos,
        world_line_y + scaled_note_x * sin,
    )
}

fn spawn_hit_effect_system(
    mut commands: Commands,
    query: Query<(&Note, &ChildOf, Entity, Option<&PlayedHitEffect>)>,
    line_query: Query<Option<&Events>>,
    line_event_query: Query<&LineEvent>,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
    assets: Res<ImageAssets>,
    paused: Res<Paused>,

    atlas_res: Res<HitEffectAtlas>,
    meta: Res<RespackMeta>,

    game_viewport: Res<GameViewport>,

    config: Res<GameConfig>,
) {
    if config.hide_hit_effect {
        return;
    }

    for (note, child_of, entity, played) in &query {
        let events = match line_query.get(child_of.parent()).ok().flatten() {
            Some(events) => events,
            None => continue,
        };

        let note_time = bpm_list.time_at(note.beat);

        // For hold notes, use current time; for other notes, use note hit time
        let effect_beat: f32 = match note.kind {
            NoteKind::Hold { .. } => bpm_list.beat_at(time.0).into(),
            _ => note.beat.value(),
        };

        let mut spawn = || {
            let (line_x, line_y, line_rotation) =
                evaluate_line_at_beat(effect_beat, events, &line_event_query);
            let position =
                compute_hit_effect_position(line_x, line_y, line_rotation, note.x, &game_viewport);

            let mut sprite = Sprite::from_atlas_image(
                assets.hit.clone(),
                TextureAtlas {
                    layout: atlas_res.layout.clone(),
                    index: 0,
                },
            );

            sprite.color = PERFECT_COLOR;

            commands.spawn((
                sprite,
                HitEffect(position),
                AnimationTimer(Timer::new(
                    Duration::from_secs_f32(meta.hit_fx.duration) / atlas_res.frame_count,
                    TimerMode::Repeating,
                )),
            ));

            let factor = game_viewport.0.width() / 426.0;

            for _ in 0..4 {
                commands.spawn(HitParticleBundle::new(position, factor));
            }

            commands.entity(entity).insert(PlayedHitEffect(time.0));
        };

        match note.kind {
            NoteKind::Hold { .. } => {
                let end_time = bpm_list.time_at(note.end_beat());

                // half-beat interval at the current BPM
                let interval = 30.0 / bpm_list.bpm_at(time.0);

                if note_time <= time.0
                    && time.0 <= end_time
                    && !paused.0
                    && (played.is_none() || played.is_some_and(|last| (time.0 - last.0) > interval))
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
    shape: Shape,
    transform: Transform,
}

impl HitParticleBundle {
    pub fn new(position: Vec2, factor: f32) -> Self {
        let size = rand::rng().random_range(7.0..=10.0) * factor;
        let shape = shapes::Rectangle {
            extents: Vec2::splat(size),
            origin: Default::default(),
            ..default()
        };

        let angle = rand::rng().random_range(-std::f32::consts::PI..=std::f32::consts::PI);
        let quat = Quat::from_rotation_z(angle);

        Self {
            hit_particle: Default::default(),
            velocity: Default::default(),
            direction: Direction(quat),
            lifetime: Default::default(),
            shape: ShapeBuilder::with(&shape).fill(PERFECT_COLOR).build(),
            transform: Transform {
                translation: position.extend(HIT_EFFECT_LAYER),
                ..default()
            },
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

fn update_opacity_system(mut query: Query<(&mut Shape, &Lifetime), With<HitParticle>>) {
    for (mut shape, lifetime) in &mut query {
        shape.fill = shape
            .fill
            .map(|fill| fill.color.with_alpha((0.5 - lifetime.0) / 0.5).into());
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
