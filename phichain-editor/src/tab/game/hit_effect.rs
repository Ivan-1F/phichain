use crate::assets::ImageAssets;
use crate::project::project_loaded;
use crate::tab::game::GameViewport;
use crate::timing::{ChartTime, Paused};
use bevy::prelude::*;
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
                update_hit_effect_system,
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

fn setup_system(
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = TextureAtlasLayout::from_grid(Vec2::splat(256.0), 1, 30, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    commands.insert_resource(TextureAtlasLayoutHandle(texture_atlas_layout.clone()));
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

fn update_hit_effect_system(
    mut query: Query<(&mut Transform, &Parent, &HitEffect)>,
    parent_query: Query<&Transform, Without<HitEffect>>,
) {
    for (mut transform, parent, effect) in &mut query {
        transform.translation = Vec3::new(effect.0.x, effect.0.y, 10.0);
        transform.rotation = parent_query.get(parent.get()).unwrap().rotation.inverse();
    }
}

fn update_hit_effect_scale_system(
    mut query: Query<&mut Transform, With<HitEffect>>,
    game_viewport: Res<GameViewport>,
) {
    for mut transform in &mut query {
        transform.scale = Vec3::splat(
            game_viewport.0.width() / 8000.0 / (game_viewport.0.width() * 3.0 / 1920.0) * 6.0,
        )
    }
}

#[derive(Component, Debug)]
struct PlayedHitEffect(f32);

fn spawn_hit_effect_system(
    mut commands: Commands,
    query: Query<(&Note, &Transform, Entity, &Parent, Option<&PlayedHitEffect>)>,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
    assets: Res<ImageAssets>,
    paused: Res<Paused>,

    texture_atlas_layout_handle: Res<TextureAtlasLayoutHandle>,
) {
    for (note, transform, entity, parent, played) in &query {
        let mut spawn = || {
            commands.entity(parent.get()).with_children(|p| {
                p.spawn((
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
                    HitEffect(Vec2::new(transform.translation.x, transform.translation.y)),
                    AnimationTimer(Timer::new(
                        Duration::from_millis(500 / 30),
                        TimerMode::Repeating,
                    )),
                ));
            });

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
