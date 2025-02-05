use bevy::{prelude::*, sprite::Anchor};
use num::{FromPrimitive, Rational32};
use phichain_assets::ImageAssets;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};
use phichain_chart::event::{EventEvaluationResult, LineEvent, LineEventKind};
use phichain_chart::line::{Line, LineOpacity, LinePosition, LineRotation};

use crate::constants::PERFECT_COLOR;
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
        .add_systems(Update, calculate_speed_events_system.in_set(GameSet))
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
            &Children,
        ),
        With<Line>,
    >,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
) {
    let beat: f32 = bpm_list.beat_at(time.0).into();
    for (mut position, mut rotation, mut opacity, mut speed, children) in &mut line_query {
        let mut x_value = EventEvaluationResult::Unaffected;
        let mut y_value = EventEvaluationResult::Unaffected;
        let mut rotation_value = EventEvaluationResult::Unaffected;
        let mut opacity_value = EventEvaluationResult::Unaffected;
        let mut speed_value = EventEvaluationResult::Unaffected;

        for event in children.iter().filter_map(|x| event_query.get(*x).ok()) {
            let value = event.evaluate(beat);
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
    }
}

pub fn update_line_system(
    mut line_query: Query<
        (
            &LinePosition,
            &LineRotation,
            &LineOpacity,
            &mut Transform,
            &mut Sprite,
            Option<&Parent>,
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
    query: Query<(&Children, Entity), With<Line>>,
    game_viewport: Res<GameViewport>,
    speed_event_query: Query<(&SpeedEvent, &LineEvent, &Parent)>,
    mut note_query: Query<(&mut Transform, &mut Sprite, &mut Visibility, &Note)>,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
) {
    let all_speed_events: Vec<_> = speed_event_query.iter().collect();
    for (children, entity) in &query {
        let mut speed_events: Vec<&SpeedEvent> = all_speed_events
            .iter()
            .filter(|(_, _, parent)| parent.get() == entity)
            .map(|(s, _, _)| *s)
            .collect();
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
pub struct HoldHead;
#[derive(Debug, Component, Default, Clone)]
pub struct HoldTail;
#[derive(Debug, Component, Default, Clone)]
pub struct HoldComponent;

#[derive(Bundle)]
struct HoldHeadBundle {
    sprite: Sprite,
    hold_head: HoldHead,
    hold_component: HoldComponent,
}

impl HoldHeadBundle {
    fn new() -> Self {
        Self {
            sprite: Sprite {
                anchor: Anchor::TopCenter,
                ..default()
            },
            hold_head: Default::default(),
            hold_component: Default::default(),
        }
    }
}

#[derive(Bundle)]
struct HoldTailBundle {
    sprite: Sprite,
    hold_tail: HoldTail,
    hold_component: HoldComponent,
}

impl HoldTailBundle {
    fn new() -> Self {
        Self {
            sprite: Sprite {
                anchor: Anchor::BottomCenter,
                ..default()
            },
            hold_tail: Default::default(),
            hold_component: Default::default(),
        }
    }
}

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
                    p.spawn(HoldHeadBundle::new());
                    p.spawn(HoldTailBundle::new());
                });
            }
            Some(children) => {
                if children.iter().all(|c| head_query.get(*c).is_err()) {
                    commands.entity(entity).with_children(|p| {
                        p.spawn(HoldHeadBundle::new());
                    });
                }
                if children.iter().all(|c| tail_query.get(*c).is_err()) {
                    commands.entity(entity).with_children(|p| {
                        p.spawn(HoldTailBundle::new());
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
    mut head_query: Query<(&mut Sprite, &Parent), (With<HoldHead>, Without<HoldTail>)>,
    mut tail_query: Query<&mut Sprite, (With<HoldTail>, Without<HoldHead>)>,
    parent_query: Query<Option<&Highlighted>>,
    assets: Res<ImageAssets>,
) {
    for (mut sprite, parent) in &mut head_query {
        if let Ok(highlight) = parent_query.get(parent.get()).map(|x| x.is_some()) {
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
    component_query: Query<(&Parent, Entity), With<HoldComponent>>,
) {
    for (parent, entity) in &component_query {
        let note = query.get(parent.get());
        if note.is_err() || note.is_ok_and(|n| !n.kind.is_hold()) {
            // despawning children does not remove references for parent
            // https://github.com/bevyengine/bevy/issues/12235
            commands.entity(parent.get()).remove_children(&[entity]);

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

#[derive(Component, Debug)]
pub struct SpeedEvent {
    start_time: f32,
    end_time: f32,
    start_value: f32,
    end_value: f32,
}

impl SpeedEvent {
    fn new(start_time: f32, end_time: f32, start_value: f32, end_value: f32) -> Self {
        Self {
            start_time,
            end_time,
            start_value,
            end_value,
        }
    }
}

pub fn calculate_speed_events_system(
    mut commands: Commands,
    query: Query<(&LineEvent, Entity)>,
    bpm_list: Res<BpmList>,
) {
    for (event, entity) in &query {
        if let LineEventKind::Speed = event.kind {
            commands.entity(entity).try_insert(SpeedEvent::new(
                bpm_list.time_at(event.start_beat),
                bpm_list.time_at(event.end_beat),
                event.value.start(),
                event.value.end(),
            ));
        }
    }
}

fn distance_at(speed_events: &Vec<&SpeedEvent>, time: f32) -> f32 {
    let mut t = 0.0;
    let mut v = 10.0;
    let mut area = 0.0;

    for event in speed_events {
        if event.start_time > t {
            let delta = ((event.start_time.min(time) - t) * v).max(0.0);
            area += delta;
        }

        let time_delta = (time.min(event.end_time) - event.start_time).max(0.0);
        if time_delta > 0.0 {
            let time_span = event.end_time - event.start_time;
            let speed_span = event.end_value - event.start_value;

            let speed_end = event.start_value + time_delta / time_span * speed_span;

            let delta = time_delta * (event.start_value + speed_end) / 2.0;
            area += delta;
        }

        t = event.end_time;
        v = event.end_value;
    }

    if time > t {
        area += (time - t) * v;
    }

    area
}
