use bevy::prelude::*;

use crate::project::Project;
use crate::{project::project_loaded, score::GameScore};

use super::GameViewport;

const UI_TEXT_MARGIN: f32 = 10.0;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BaseTextScale(1.0))
            .add_systems(
                Update,
                (update_base_text_scale_system, update_text_scale_system)
                    .chain()
                    .run_if(project_loaded()),
            )
            // combo
            .add_systems(Startup, setup_combo_ui_system)
            .add_systems(Update, update_combo_system.run_if(project_loaded()))
            .add_systems(Update, hide_combo_below_3_system.run_if(project_loaded()))
            // score
            .add_systems(Startup, spawn_score_ui_system)
            .add_systems(Update, update_score_system.run_if(project_loaded()))
            // name
            .add_systems(Startup, spawn_name_ui_system)
            .add_systems(Update, update_name_system.run_if(project_loaded()))
            // level
            .add_systems(Startup, spawn_level_ui_system)
            .add_systems(Update, update_level_system.run_if(project_loaded()));
    }
}

/// Scale based on [BaseTextScale] for a specific text
#[derive(Component, Debug)]
struct TextScale(f32);

/// Base game ui base text scale
#[derive(Resource, Debug)]
struct BaseTextScale(f32);

fn update_base_text_scale_system(
    mut scale: ResMut<BaseTextScale>,
    game_viewport: Res<GameViewport>,
) {
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
                top: Val::Px(UI_TEXT_MARGIN),
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
                    TextScale(1.0),
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
                                },
                            ),
                            ..default()
                        },
                        ComboIndicator,
                        TextScale(0.4),
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
                position_type: PositionType::Absolute,
                top: Val::Px(UI_TEXT_MARGIN),
                right: Val::Px(UI_TEXT_MARGIN),
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
                        },
                    ),
                    ..default()
                },
                ScoreText,
                TextScale(0.8),
            ));
        });
}

/// Marker component to represent the name text
#[derive(Component)]
struct NameText;

fn spawn_name_ui_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(UI_TEXT_MARGIN),
                bottom: Val::Px(UI_TEXT_MARGIN),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text::from_section(
                        "Name",
                        TextStyle {
                            font: asset_server.load("font/phigros.ttf"),
                            font_size: 10.0,
                            color: Color::WHITE,
                        },
                    ),
                    ..default()
                },
                NameText,
                TextScale(0.5),
            ));
        });
}

/// Marker component to represent the level text
#[derive(Component)]
struct LevelText;

fn spawn_level_ui_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(UI_TEXT_MARGIN),
                right: Val::Px(UI_TEXT_MARGIN),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text::from_section(
                        "Level",
                        TextStyle {
                            font: asset_server.load("font/phigros.ttf"),
                            font_size: 10.0,
                            color: Color::WHITE,
                        },
                    ),
                    ..default()
                },
                LevelText,
                TextScale(0.5),
            ));
        });
}

fn update_text_scale_system(scale: Res<BaseTextScale>, mut query: Query<(&mut Text, &TextScale)>) {
    for (mut text, text_scale) in &mut query {
        text.sections[0].style.font_size = scale.0 * 1.32 * text_scale.0;
    }
}

fn update_combo_system(mut text_query: Query<&mut Text, With<ComboText>>, score: Res<GameScore>) {
    let mut combo_text = text_query.single_mut();
    combo_text.sections[0].value = score.combo().to_string();
}

fn hide_combo_below_3_system(
    mut combo_query: Query<&mut Visibility, With<Combo>>,
    score: Res<GameScore>,
) {
    let mut visibility = combo_query.single_mut();
    *visibility = if score.combo() >= 3 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
}

fn update_score_system(
    mut score_text_query: Query<&mut Text, With<ScoreText>>,
    score: Res<GameScore>,
) {
    let mut score_text = score_text_query.single_mut();
    score_text.sections[0].value = score.score_text();
}

fn update_name_system(
    mut name_text_query: Query<&mut Text, With<NameText>>,
    project: Res<Project>,
) {
    let mut name_text = name_text_query.single_mut();
    name_text.sections[0].value = project.meta.name.replace(' ', "\u{00A0}");
}

fn update_level_system(
    mut name_text_query: Query<&mut Text, With<LevelText>>,
    project: Res<Project>,
) {
    let mut name_text = name_text_query.single_mut();
    name_text.sections[0].value = project.meta.level.replace(' ', "\u{00A0}");
}
