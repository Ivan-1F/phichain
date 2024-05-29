use bevy::{prelude::*, sprite::Anchor};
use num::{FromPrimitive, Rational32};
use phichain_chart::event::{LineEvent, LineEventKind};
use phichain_chart::line::{Line, LineOpacity, LinePosition, LineRotation};

use crate::editing::pending::Pending;
use crate::highlight::Highlighted;
use crate::selection::Selected;
use crate::{
    assets::ImageAssets,
    constants::{CANVAS_HEIGHT, CANVAS_WIDTH},
    project::project_loaded,
    timing::{BpmList, ChartTime},
};
use phichain_chart::line::LineSpeed;
use phichain_chart::note::{Note, NoteKind};

use super::{GameCamera, GameViewport};

pub struct CoreGamePlugin;

impl Plugin for CoreGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, zoom_scale_system.run_if(project_loaded()))
            .add_systems(
                // note placement runs on Update, we need to edit them after they are being spawned into the world
                // FIXME: use SystemSet for better schedule control
                PostUpdate,
                (
                    update_note_scale_system,
                    update_note_system,
                    update_note_y_system,
                    update_note_texture_system,
                )
                    .chain()
                    .run_if(project_loaded()),
            )
            .add_systems(
                PostUpdate,
                (compute_line_system, update_line_system)
                    .chain()
                    .run_if(project_loaded()),
            )
            .add_systems(
                PostUpdate,
                (update_line_texture_system, update_note_texture_system).run_if(project_loaded()),
            )
            .add_systems(PostUpdate, update_note_tint_system.run_if(project_loaded()))
            .add_systems(
                PostUpdate,
                calculate_speed_events_system.run_if(project_loaded()),
            );
    }
}

fn zoom_scale_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut OrthographicProjection, With<GameCamera>>,
) {
    let mut projection = query.single_mut();
    if keyboard.pressed(KeyCode::KeyI) {
        projection.scale /= 1.01;
    } else if keyboard.pressed(KeyCode::KeyO) {
        projection.scale *= 1.01;
    }
}

fn update_note_scale_system(
    mut query: Query<&mut Transform, With<Note>>,
    game_viewport: Res<GameViewport>,
) {
    for mut transform in &mut query {
        transform.scale =
            Vec3::splat(game_viewport.0.width() / 8000.0 / (game_viewport.0.width() * 3.0 / 1920.0))
    }
}

fn update_note_system(
    mut query: Query<(&mut Transform, &mut Visibility, &Note)>,
    game_viewport: Res<GameViewport>,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
) {
    let beat = bpm_list.beat_at(time.0);
    for (mut transform, mut visibility, note) in &mut query {
        transform.translation.x = (note.x / CANVAS_WIDTH) * game_viewport.0.width()
            / (game_viewport.0.width() * 3.0 / 1920.0);

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

fn compute_line_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    event_query: Query<(&LineEvent, &Parent)>,
    mut line_query: Query<
        (
            &mut LinePosition,
            &mut LineRotation,
            &mut LineOpacity,
            &mut LineSpeed,
            Entity,
        ),
        With<Line>,
    >,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
) {
    let beat: f32 = bpm_list.beat_at(time.0).into();
    for (mut position, mut rotation, mut opacity, mut speed, entity) in &mut line_query {
        let mut events: Vec<_> = event_query
            .iter()
            .filter(|(_, parent)| parent.get() == entity)
            .collect();
        events.sort_by_key(|(e, _)| e.start_beat);
        for (event, _) in events {
            let value = event.evaluate(beat);
            if let Some(value) = value {
                match event.kind {
                    LineEventKind::X => position.0.x = value,
                    LineEventKind::Y => position.0.y = value,
                    LineEventKind::Rotation => rotation.0 = value.to_radians(),
                    LineEventKind::Opacity => {
                        if keyboard.pressed(KeyCode::KeyT) {
                            opacity.0 = 1.0;
                        } else {
                            opacity.0 = value / 255.0;
                        }
                    }
                    LineEventKind::Speed => speed.0 = value,
                }
            }
        }
    }
}

fn update_line_system(
    mut line_query: Query<
        (
            &LinePosition,
            &LineRotation,
            &LineOpacity,
            &mut Transform,
            &mut Sprite,
        ),
        With<Line>,
    >,
    game_viewport: Res<GameViewport>,
) {
    for (position, rotation, opacity, mut transform, mut sprite) in &mut line_query {
        transform.scale = Vec3::splat(game_viewport.0.width() * 3.0 / 1920.0);
        transform.translation.x = position.0.x / CANVAS_WIDTH * game_viewport.0.width();
        transform.translation.y = position.0.y / CANVAS_HEIGHT * game_viewport.0.height();
        transform.rotation = Quat::from_rotation_z(rotation.0);
        sprite.color.set_a(opacity.0);
    }
}

fn update_note_y_system(
    query: Query<(&Children, Entity), With<Line>>,
    game_viewport: Res<GameViewport>,
    speed_event_query: Query<(&SpeedEvent, &LineEvent, &Parent)>,
    mut note_query: Query<(&mut Transform, &mut Sprite, &Note)>,
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
            if let Ok((mut transform, mut sprite, note)) = note_query.get_mut(*child) {
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
                    }
                    _ => {
                        sprite.anchor = Anchor::Center;
                        transform.rotation = Quat::from_rotation_z(0.0_f32.to_radians());
                    }
                }

                transform.translation.y = y * if note.above { 1.0 } else { -1.0 };
            }
        }
    }
}

fn update_note_texture_system(
    mut query: Query<(&mut Handle<Image>, &Note, Option<&Highlighted>)>,
    assets: Res<ImageAssets>,
) {
    for (mut image, note, highlighted) in &mut query {
        match (note.kind, highlighted.is_some()) {
            (NoteKind::Tap, true) => *image = assets.tap_highlight.clone(),
            (NoteKind::Drag, true) => *image = assets.drag_highlight.clone(),
            (NoteKind::Hold { .. }, true) => *image = assets.hold_highlight.clone(),
            (NoteKind::Flick, true) => *image = assets.flick_highlight.clone(),
            (NoteKind::Tap, false) => *image = assets.tap.clone(),
            (NoteKind::Drag, false) => *image = assets.drag.clone(),
            (NoteKind::Hold { .. }, false) => *image = assets.hold.clone(),
            (NoteKind::Flick, false) => *image = assets.flick.clone(),
        }
    }
}

fn update_note_tint_system(
    mut query: Query<(&mut Sprite, Option<&Selected>, Option<&Pending>), With<Note>>,
) {
    for (mut sprite, selected, pending) in &mut query {
        let tint = if selected.is_some() {
            Color::LIME_GREEN
        } else {
            Color::WHITE
        };
        let alpha = if pending.is_some() { 40.0 / 255.0 } else { 1.0 };
        sprite.color = tint.with_a(alpha);
    }
}

fn update_line_texture_system(
    mut query: Query<&mut Handle<Image>, With<Line>>,
    assets: Res<ImageAssets>,
) {
    for mut image in &mut query {
        *image = assets.line.clone();
    }
}

#[derive(Component, Debug)]
struct SpeedEvent {
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

fn calculate_speed_events_system(
    mut commands: Commands,
    query: Query<(&LineEvent, Entity)>,
    bpm_list: Res<BpmList>,
) {
    for (event, entity) in &query {
        if let LineEventKind::Speed = event.kind {
            commands.entity(entity).insert(SpeedEvent::new(
                bpm_list.time_at(event.start_beat),
                bpm_list.time_at(event.end_beat),
                event.start,
                event.end,
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
