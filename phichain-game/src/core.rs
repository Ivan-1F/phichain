use bevy::{prelude::*, sprite::Anchor};
use phichain_assets::{HoldParts, ImageAssets, RespackDimensions};
use phichain_chart::bpm_list::BpmList;
use phichain_chart::constants::{CANVAS_HEIGHT, CANVAS_WIDTH};
use phichain_chart::event::LineEventKind;
use phichain_chart::line::{Line, LineOpacity, LinePosition, LineRotation};

use crate::constants::PERFECT_COLOR;
use crate::event::EventIndex;
use crate::highlight::Highlighted;
use crate::layer::{HOLD_LAYER, NOTE_LAYER};
use crate::scale;
use crate::speed::SpeedTable;
use crate::{ChartTime, Derived, GameConfig, GameSet, GameViewport};
use bevy::platform::collections::HashMap;
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
        .add_systems(Update, update_line_texture_system.in_set(GameSet))
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
                .chain()
                .in_set(GameSet),
        );
    }
}

pub fn update_note_scale_system(
    mut query: Query<&mut Transform, With<Note>>,
    game_viewport: Res<GameViewport>,
    config: Res<GameConfig>,
    dimensions: Res<RespackDimensions>,
) {
    let scale = scale::note_sprite_local_scale(
        game_viewport.0.width(),
        dimensions.note_width,
        config.note_scale,
    );
    for mut transform in &mut query {
        transform.scale = Vec3::splat(scale);
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
            / scale::line_world_scale(game_viewport.0.width());

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
    index: Res<EventIndex>,
    mut line_query: Query<
        (
            Entity,
            &mut LinePosition,
            &mut LineRotation,
            &mut LineOpacity,
            &mut LineSpeed,
        ),
        With<Line>,
    >,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
) {
    let beat: f32 = bpm_list.beat_at(time.0).into();
    line_query.par_iter_mut().for_each(
        |(line, mut position, mut rotation, mut opacity, mut speed)| {
            let value = |kind| index.evaluate(line, kind, beat);

            if let Some(x_value) = value(LineEventKind::X) {
                position.0.x = x_value;
            }
            if let Some(y_value) = value(LineEventKind::Y) {
                position.0.y = y_value;
            }
            if let Some(rotation_value) = value(LineEventKind::Rotation) {
                rotation.0 = rotation_value.to_radians();
            }
            if keyboard.pressed(KeyCode::KeyT) {
                opacity.0 = 1.0;
            } else if let Some(opacity_value) = value(LineEventKind::Opacity) {
                opacity.0 = opacity_value / 255.0;
            }
            if let Some(speed_value) = value(LineEventKind::Speed) {
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
        let scale = scale::line_world_scale(game_viewport.0.width());
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
    query: Query<(Entity, &Children), With<Line>>,
    game_viewport: Res<GameViewport>,
    mut note_query: Query<(
        &mut Transform,
        &mut Anchor,
        &mut Visibility,
        &Note,
        Option<&Highlighted>,
    )>,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
    dimensions: Res<RespackDimensions>,
    index: Res<EventIndex>,
    mut tables: Local<HashMap<Entity, SpeedTable>>,
) {
    // speed events rarely change, so their distance tables are only rebuilt then
    if index.is_changed() || bpm_list.is_changed() {
        tables.clear();
        for (line, _) in &query {
            if let Some(track) = index.track(line, LineEventKind::Speed) {
                tables.insert(line, SpeedTable::new(track, &bpm_list));
            }
        }
    }

    let no_speed_events = SpeedTable::default();
    for (line, children) in &query {
        let table = tables.get(&line).unwrap_or(&no_speed_events);
        let distance = |time| {
            table.distance_at(time) * (game_viewport.0.height() * (120.0 / 900.0))
                / scale::line_world_scale(game_viewport.0.width())
        };
        let current_distance = distance(time.0);
        for child in children {
            if let Ok((mut transform, mut anchor, mut visibility, note, highlighted)) =
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
                        *anchor = Anchor::BOTTOM_CENTER;
                        transform.rotation = Quat::from_rotation_z(
                            if note.above { 0.0_f32 } else { 180.0_f32 }.to_radians(),
                        );
                        let body_height = if highlighted.is_some() {
                            dimensions.hold_highlight_body_height
                        } else {
                            dimensions.hold_body_height
                        };
                        transform.scale.y = height / body_height;

                        // hide notes behind line (cover)
                        if height < 0.0 {
                            *visibility = Visibility::Hidden;
                        }
                    }
                    _ => {
                        *anchor = Anchor::CENTER;
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
    hold_parts: Res<HoldParts>,
) {
    for (mut sprite, note, highlighted) in &mut query {
        match (note.kind, highlighted.is_some()) {
            (NoteKind::Tap, true) => sprite.image = assets.tap_highlight.clone(),
            (NoteKind::Drag, true) => sprite.image = assets.drag_highlight.clone(),
            (NoteKind::Hold { .. }, true) => sprite.image = hold_parts.body_highlight.clone(),
            (NoteKind::Flick, true) => sprite.image = assets.flick_highlight.clone(),
            (NoteKind::Tap, false) => sprite.image = assets.tap.clone(),
            (NoteKind::Drag, false) => sprite.image = assets.drag.clone(),
            (NoteKind::Hold { .. }, false) => sprite.image = hold_parts.body.clone(),
            (NoteKind::Flick, false) => sprite.image = assets.flick.clone(),
        }
    }
}

#[derive(Debug, Component, Default, Clone)]
#[require(Sprite, Anchor::TOP_CENTER, HoldComponent)]
pub struct HoldHead;
#[derive(Debug, Component, Default, Clone)]
#[require(Sprite, Anchor::BOTTOM_CENTER, HoldComponent)]
pub struct HoldTail;
#[derive(Debug, Component, Default, Clone)]
#[require(Derived)]
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
    parent_query: Query<
        (&Transform, &Children, Option<&Highlighted>),
        (Without<HoldHead>, Without<HoldTail>),
    >,
    dimensions: Res<RespackDimensions>,
) {
    for (transform, children, highlighted) in &parent_query {
        let body_height = if highlighted.is_some() {
            dimensions.hold_highlight_body_height
        } else {
            dimensions.hold_body_height
        };
        for child in children {
            if let Ok(mut head) = head_query.get_mut(*child) {
                head.scale.y = 1.0 / transform.scale.y * transform.scale.x;
            }
            if let Ok(mut tail) = tail_query.get_mut(*child) {
                tail.scale.y = 1.0 / transform.scale.y * transform.scale.x;
                tail.translation.y = body_height;
            }
        }
    }
}

pub fn update_hold_component_texture_system(
    mut head_query: Query<(&mut Sprite, &ChildOf), (With<HoldHead>, Without<HoldTail>)>,
    mut tail_query: Query<(&mut Sprite, &ChildOf), (With<HoldTail>, Without<HoldHead>)>,
    parent_query: Query<Option<&Highlighted>>,
    hold_parts: Res<HoldParts>,
) {
    for (mut sprite, child_of) in &mut head_query {
        if let Ok(highlight) = parent_query.get(child_of.parent()).map(|x| x.is_some()) {
            sprite.image = if highlight {
                hold_parts.head_highlight.clone()
            } else {
                hold_parts.head.clone()
            };
        }
    }
    for (mut sprite, child_of) in &mut tail_query {
        if let Ok(highlight) = parent_query.get(child_of.parent()).map(|x| x.is_some()) {
            sprite.image = if highlight {
                hold_parts.tail_highlight.clone()
            } else {
                hold_parts.tail.clone()
            };
        }
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
                .detach_children(&[entity]);

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
