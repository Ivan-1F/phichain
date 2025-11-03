use bevy::{prelude::*, sprite::Anchor};
use num::{FromPrimitive, Rational32};
use phichain_assets::ImageAssets;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};
use phichain_chart::event::{EventEvaluationResult, LineEvent, LineEventKind};
use phichain_chart::line::{Line, LineOpacity, LinePosition, LineRotation};

use crate::constants::PERFECT_COLOR;
use crate::event::Events;
use crate::highlight::Highlighted;
use crate::layer::{HOLD_LAYER, NOTE_LAYER};
use crate::scale::NoteScale;
use crate::{ChartTime, GameConfig, GameSet, GameViewport};
use phichain_chart::line::LineSpeed;
use phichain_chart::note::{Note, NoteKind};

pub struct CoreGamePlugin;

impl Plugin for CoreGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            // note placement runs on Update, we need to edit them after they are being spawned into the world
            Update,
            (
                update_note_scale_system,
                update_note_system,
                update_note_y_system,
                update_note_texture_system,
            )
                .chain()
                .in_set(GameSet),
        )
        .add_systems(
            Update,
            (compute_line_system, update_line_system)
                .chain()
                .in_set(GameSet),
        )
        .add_systems(
            Update,
            (update_line_texture_system, update_note_texture_system).in_set(GameSet),
        )
        // hold components
        .add_systems(
            Update,
            (
                spawn_hold_component_system,
                update_hold_components_scale_system
                    // otherwise heads & tails will keep twitching
                    .after(update_note_y_system),
                update_hold_component_texture_system,
                hide_hold_head_system,
                despawn_hold_component_system,
            )
                .in_set(GameSet),
        );
    }
}

pub fn update_note_scale_system(
    mut query: Query<&mut Transform, With<Note>>,
    game_viewport: Res<GameViewport>,
    note_scale: Res<NoteScale>,
) {
    for mut transform in &mut query {
        transform.scale = Vec3::splat(note_scale.0 / (game_viewport.0.width() * 3.0 / 1920.0))
    }
}

pub fn update_note_system(
    mut query: Query<(&mut Transform, &mut Visibility, &Note)>,
    game_viewport: Res<GameViewport>,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
) {
    let beat = bpm_list.beat_at(time.0);
    for (mut transform, mut visibility, note) in &mut query {
        transform.translation.x = (note.x / CANVAS_WIDTH) * game_viewport.0.width()
            / (game_viewport.0.width() * 3.0 / 1920.0);

        transform.translation.z = match note.kind {
            NoteKind::Hold { .. } => HOLD_LAYER,
            _ => NOTE_LAYER,
        };

        let hold_beat = match note.kind {
            NoteKind::Hold { hold_beat } => hold_beat.value(),
            _ => 0.0,
        };
        *visibility = if note.beat.value() + hold_beat < beat.into() {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}

pub fn compute_line_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    event_query: Query<&LineEvent>,
    mut line_query: Query<
        (
            &mut LinePosition,
            &mut LineRotation,
            &mut LineOpacity,
            &mut LineSpeed,
            &Events,
        ),
        With<Line>,
    >,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
) {
    let beat: f32 = bpm_list.beat_at(time.0).into();
    line_query.par_iter_mut().for_each(
        |(mut position, mut rotation, mut opacity, mut speed, events)| {
            let mut x_value = EventEvaluationResult::Unaffected;
            let mut y_value = EventEvaluationResult::Unaffected;
            let mut rotation_value = EventEvaluationResult::Unaffected;
            let mut opacity_value = EventEvaluationResult::Unaffected;
            let mut speed_value = EventEvaluationResult::Unaffected;

            for event in events.iter().filter_map(|x| event_query.get(x).ok()) {
                let value = event.evaluate_inclusive(beat);
                match event.kind {
                    LineEventKind::X => x_value = x_value.max(value),
                    LineEventKind::Y => y_value = y_value.max(value),
                    LineEventKind::Rotation => rotation_value = rotation_value.max(value),
                    LineEventKind::Opacity => opacity_value = opacity_value.max(value),
                    LineEventKind::Speed => speed_value = speed_value.max(value),
                }
            }

            if let Some(x_value) = x_value.value() {
                position.0.x = x_value;
            }
            if let Some(y_value) = y_value.value() {
                position.0.y = y_value;
            }
            if let Some(rotation_value) = rotation_value.value() {
                rotation.0 = rotation_value.to_radians();
            }
            if keyboard.pressed(KeyCode::KeyT) {
                opacity.0 = 1.0;
            } else if let Some(opacity_value) = opacity_value.value() {
                opacity.0 = opacity_value / 255.0;
            }
            if let Some(speed_value) = speed_value.value() {
                speed.0 = speed_value;
            }
        },
    );
}

pub fn update_line_system(
    mut line_query: Query<
        (
            &LinePosition,
            &LineRotation,
            &LineOpacity,
            &mut Transform,
            &mut Sprite,
            Option<&ChildOf>,
        ),
        With<Line>,
    >,
    game_viewport: Res<GameViewport>,

    config: Res<GameConfig>,
) {
    for (position, rotation, opacity, mut transform, mut sprite, parent) in &mut line_query {
        let scale = game_viewport.0.width() * 3.0 / 1920.0;
        transform.scale = Vec3::splat(if parent.is_some() { 1.0 } else { scale });
        transform.translation.x = position.0.x / CANVAS_WIDTH * game_viewport.0.width()
            / if parent.is_some() { scale } else { 1.0 };
        transform.translation.y = position.0.y / CANVAS_HEIGHT * game_viewport.0.height()
            / if parent.is_some() { scale } else { 1.0 };
        transform.rotation = Quat::from_rotation_z(rotation.0);

        sprite.color = if config.fc_ap_indicator {
            PERFECT_COLOR
        } else {
            Color::WHITE
        }
        .with_alpha(opacity.0);
    }
}

pub fn update_note_y_system(
    query: Query<(&Children, Option<&Events>), With<Line>>,
    game_viewport: Res<GameViewport>,
    line_event_query: Query<&LineEvent>,
    mut note_query: Query<(&mut Transform, &mut Sprite, &mut Visibility, &Note)>,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
) {
    for (children, events) in &query {
        let mut speed_events: Vec<SpeedSegment> = Vec::new();
        if let Some(events) = events {
            for event_entity in events.iter() {
                if let Ok(event) = line_event_query.get(event_entity) {
                    if event.kind.is_speed() {
                        speed_events.push(SpeedSegment {
                            start_time: bpm_list.time_at(event.start_beat),
                            end_time: bpm_list.time_at(event.end_beat),
                            start_value: event.value.start(),
                            end_value: event.value.end(),
                        });
                    }
                }
            }
        }

        speed_events.sort_by(|a, b| {
            Rational32::from_f32(a.start_time).cmp(&Rational32::from_f32(b.start_time))
        });

        let distance = |time| {
            distance_at(&speed_events, time) * (game_viewport.0.height() * (120.0 / 900.0))
                / (game_viewport.0.width() * 3.0 / 1920.0)
        };
        let current_distance = distance(time.0);
        for child in children {
            if let Ok((mut transform, mut sprite, mut visibility, note)) =
                note_query.get_mut(*child)
            {
                let mut y = (distance(bpm_list.time_at(note.beat)) - current_distance) * note.speed;
                match note.kind {
                    NoteKind::Hold { hold_beat } => {
                        y = y.max(0.0);
                        let height = (distance(bpm_list.time_at(note.beat + hold_beat))
                            - current_distance)
                            * note.speed
                            - y;
                        sprite.anchor = Anchor::BottomCenter;
                        transform.rotation = Quat::from_rotation_z(
                            if note.above { 0.0_f32 } else { 180.0_f32 }.to_radians(),
                        );
                        transform.scale.y = height / 1900.0;

                        // hide notes behind line (cover)
                        if height < 0.0 {
                            *visibility = Visibility::Hidden;
                        }
                    }
                    _ => {
                        sprite.anchor = Anchor::Center;
                        transform.rotation = Quat::from_rotation_z(0.0_f32.to_radians());

                        // hide notes behind line (cover)
                        if y < 0.0 {
                            *visibility = Visibility::Hidden;
                        }
                    }
                }

                transform.translation.y = y * if note.above { 1.0 } else { -1.0 };
            }
        }
    }
}

pub fn update_note_texture_system(
    mut query: Query<(&mut Sprite, &Note, Option<&Highlighted>)>,
    assets: Res<ImageAssets>,
) {
    for (mut sprite, note, highlighted) in &mut query {
        match (note.kind, highlighted.is_some()) {
            (NoteKind::Tap, true) => sprite.image = assets.tap_highlight.clone(),
            (NoteKind::Drag, true) => sprite.image = assets.drag_highlight.clone(),
            (NoteKind::Hold { .. }, true) => sprite.image = assets.hold_highlight.clone(),
            (NoteKind::Flick, true) => sprite.image = assets.flick_highlight.clone(),
            (NoteKind::Tap, false) => sprite.image = assets.tap.clone(),
            (NoteKind::Drag, false) => sprite.image = assets.drag.clone(),
            (NoteKind::Hold { .. }, false) => sprite.image = assets.hold.clone(),
            (NoteKind::Flick, false) => sprite.image = assets.flick.clone(),
        }
    }
}

#[derive(Debug, Component, Default, Clone)]
#[require(
    Sprite {
        anchor: Anchor::TopCenter,
        ..default()
    },
    HoldComponent,
)]
pub struct HoldHead;
#[derive(Debug, Component, Default, Clone)]
#[require(
    Sprite {
        anchor: Anchor::BottomCenter,
        ..default()
    },
    HoldComponent,
)]
pub struct HoldTail;
#[derive(Debug, Component, Default, Clone)]
pub struct HoldComponent;

pub fn spawn_hold_component_system(
    mut commands: Commands,
    query: Query<(Option<&Children>, Entity, &Note)>,
    head_query: Query<&HoldHead>,
    tail_query: Query<&HoldTail>,
) {
    for (children, entity, note) in &query {
        if !note.kind.is_hold() {
            continue;
        }

        match children {
            None => {
                commands.entity(entity).with_children(|p| {
                    p.spawn(HoldHead);
                    p.spawn(HoldTail);
                });
            }
            Some(children) => {
                if children.iter().all(|c| head_query.get(c).is_err()) {
                    commands.entity(entity).with_children(|p| {
                        p.spawn(HoldHead);
                    });
                }
                if children.iter().all(|c| tail_query.get(c).is_err()) {
                    commands.entity(entity).with_children(|p| {
                        p.spawn(HoldTail);
                    });
                }
            }
        }
    }
}

pub fn update_hold_components_scale_system(
    mut head_query: Query<&mut Transform, (With<HoldHead>, Without<HoldTail>)>,
    mut tail_query: Query<&mut Transform, (With<HoldTail>, Without<HoldHead>)>,
    parent_query: Query<(&Transform, &Children), (Without<HoldHead>, Without<HoldTail>)>,
) {
    for (transform, children) in &parent_query {
        for child in children {
            if let Ok(mut head) = head_query.get_mut(*child) {
                head.scale.y = 1.0 / transform.scale.y * transform.scale.x;
            }
            if let Ok(mut tail) = tail_query.get_mut(*child) {
                tail.scale.y = 1.0 / transform.scale.y * transform.scale.x;
                tail.translation.y = 1900.0;
            }
        }
    }
}

pub fn update_hold_component_texture_system(
    mut head_query: Query<(&mut Sprite, &ChildOf), (With<HoldHead>, Without<HoldTail>)>,
    mut tail_query: Query<&mut Sprite, (With<HoldTail>, Without<HoldHead>)>,
    parent_query: Query<Option<&Highlighted>>,
    assets: Res<ImageAssets>,
) {
    for (mut sprite, child_of) in &mut head_query {
        if let Ok(highlight) = parent_query.get(child_of.parent()).map(|x| x.is_some()) {
            sprite.image = if highlight {
                assets.hold_head_highlight.clone()
            } else {
                assets.hold_head.clone()
            };
        }
    }
    for mut sprite in &mut tail_query {
        sprite.image = assets.hold_tail.clone();
    }
}

fn hide_hold_head_system(
    note_query: Query<(&Note, &Children)>,
    mut head_query: Query<&mut Visibility, With<HoldHead>>,

    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
) {
    let beat = bpm_list.beat_at(time.0);
    for (note, children) in &note_query {
        for child in children {
            if let Ok(mut visibility) = head_query.get_mut(*child) {
                *visibility = if note.beat <= beat {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                };
            }
        }
    }
}

pub fn despawn_hold_component_system(
    mut commands: Commands,
    query: Query<&Note>,
    component_query: Query<(&ChildOf, Entity), With<HoldComponent>>,
) {
    for (child_of, entity) in &component_query {
        let note = query.get(child_of.parent());
        if note.is_err() || note.is_ok_and(|n| !n.kind.is_hold()) {
            // despawning children does not remove references for parent
            // https://github.com/bevyengine/bevy/issues/12235
            commands
                .entity(child_of.parent())
                .remove_children(&[entity]);

            commands.entity(entity).despawn();
        }
    }
}

pub fn update_line_texture_system(
    mut query: Query<&mut Sprite, With<Line>>,
    assets: Res<ImageAssets>,
) {
    for mut sprite in &mut query {
        sprite.image = assets.line.clone();
    }
}

#[derive(Debug, Clone)]
struct SpeedSegment {
    start_time: f32,
    end_time: f32,
    start_value: f32,
    end_value: f32,
}

fn distance_at(speed_events: &[SpeedSegment], time: f32) -> f32 {
    let mut last_time = 0.0;
    let mut last_speed = 10.0;
    let mut area = 0.0;

    for event in speed_events {
        if time <= last_time {
            break;
        }

        let gap_end = event.start_time.min(time);
        if gap_end > last_time {
            area += (gap_end - last_time) * last_speed;
            last_time = gap_end;
        }

        if time <= event.start_time {
            break;
        }

        let time_span = event.end_time - event.start_time;
        if time_span <= 0.0 {
            last_time = last_time.max(event.end_time);
            last_speed = event.end_value;
            continue;
        }

        let seg_start = event.start_time.max(last_time);
        let seg_end = event.end_time.min(time);
        if seg_end > seg_start {
            let speed_span = event.end_value - event.start_value;
            let start_ratio = (seg_start - event.start_time) / time_span;
            let end_ratio = (seg_end - event.start_time) / time_span;
            let start_speed = event.start_value + start_ratio * speed_span;
            let end_speed = event.start_value + end_ratio * speed_span;

            area += (seg_end - seg_start) * (start_speed + end_speed) / 2.0;

            if time <= event.end_time {
                return area;
            }
        }

        last_time = event.end_time;
        last_speed = event.end_value;
    }

    if time > last_time {
        area += (time - last_time) * last_speed;
    }

    area
}
