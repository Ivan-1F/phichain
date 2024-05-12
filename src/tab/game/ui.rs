use bevy::prelude::*;

use crate::{
    chart::note::Note,
    timing::{BpmList, ChartTime},
};

use super::GameViewport;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TextScale(1.0))
            .add_systems(Update, update_text_scale_system)
            .add_systems(Startup, setup_combo_ui_system)
            .add_systems(Update, update_combo_system)
            .add_systems(Update, hide_combo_below_3_system)
            .add_systems(Update, update_combo_text_scale_system)
            .add_systems(Startup, spawn_score_ui_system)
            .add_systems(Update, update_score_text_scale_system)
            .add_systems(Update, update_score_system);
    }
}

/// Base game ui text scale
#[derive(Resource, Debug)]
struct TextScale(f32);

fn update_text_scale_system(mut scale: ResMut<TextScale>, game_viewport: Res<GameViewport>) {
    scale.0 = if game_viewport.0.width() > game_viewport.0.height() * 0.75 {
        game_viewport.0.height() / 18.75
    } else {
        game_viewport.0.width() / 14.0625
    };
}

/// Marker component to represent the combo number text
#[derive(Component, Debug)]
struct ComboText;

/// Marker component to represent the "COMBO" text
#[derive(Component, Debug)]
struct ComboIndicator;

/// Marker component to represent the combo container
#[derive(Component, Debug)]
struct Combo;

fn setup_combo_ui_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                top: Val::Px(8.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            align_self: AlignSelf::Center,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    },
                    Combo,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle {
                            text: Text::from_section(
                                "COMBO", // this will be replaced every frame at update_combo_system
                                TextStyle {
                                    font: asset_server.load("font/phigros.ttf"),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            ),
                            ..default()
                        },
                        ComboText,
                    ));

                    parent.spawn((
                        TextBundle {
                            text: Text::from_section(
                                "COMBO",
                                TextStyle {
                                    font: asset_server.load("font/phigros.ttf"),
                                    font_size: 10.0,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            ),
                            ..default()
                        },
                        ComboIndicator,
                    ));
                });
        });
}

/// Marker component to represent the score text
#[derive(Component)]
struct ScoreText;

fn spawn_score_ui_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::FlexStart,
                top: Val::Px(8.0),
                right: Val::Px(8.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text::from_section(
                        "0000000",
                        TextStyle {
                            font: asset_server.load("font/phigros.ttf"),
                            font_size: 10.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                    ..default()
                },
                ScoreText,
            ));
        });
}

fn update_combo_system(
    mut text_query: Query<&mut Text, With<ComboText>>,
    note_query: Query<&Note>,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
) {
    let mut combo_text = text_query.single_mut();
    let combo = note_query
        .iter()
        .filter(|note| bpm_list.time_at(note.beat) <= time.0)
        .collect::<Vec<_>>()
        .len();
    combo_text.sections[0].value = combo.to_string();
}

fn hide_combo_below_3_system(
    mut combo_query: Query<&mut Visibility, With<Combo>>,
    note_query: Query<&Note>,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
) {
    let mut visibility = combo_query.single_mut();
    let combo = note_query
        .iter()
        .filter(|note| bpm_list.time_at(note.beat) <= time.0)
        .collect::<Vec<_>>()
        .len();
    *visibility = if combo >= 3 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
}

fn update_combo_text_scale_system(
    mut combo_text_query: Query<&mut Text, (With<ComboText>, Without<ComboIndicator>)>,
    mut combo_indicator_query: Query<&mut Text, (With<ComboIndicator>, Without<ComboText>)>,
    scale: Res<TextScale>,
) {
    let mut combo_text = combo_text_query.single_mut();
    let mut combo_indicator = combo_indicator_query.single_mut();
    combo_text.sections[0].style.font_size = scale.0 * 1.32;
    combo_indicator.sections[0].style.font_size = scale.0 * 1.32 * 0.4;
}

fn update_score_text_scale_system(
    mut score_text_query: Query<&mut Text, With<ScoreText>>,
    scale: Res<TextScale>,
) {
    let mut score_text = score_text_query.single_mut();
    score_text.sections[0].style.font_size = scale.0 * 1.32 * 0.8;
}

fn update_score_system(
    mut score_text_query: Query<&mut Text, With<ScoreText>>,
    note_query: Query<&Note>,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
) {
    let notes: Vec<_> = note_query.iter().collect();
    let combo = notes
        .iter()
        .filter(|note| bpm_list.time_at(note.beat) <= time.0)
        .collect::<Vec<_>>()
        .len();

    let score = 100_0000.0 * combo as f32 / notes.len() as f32;

    let mut score_text = score_text_query.single_mut();
    score_text.sections[0].value = format!("{:07}", score.round());
}
