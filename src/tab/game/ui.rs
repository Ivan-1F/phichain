use bevy::prelude::*;

use crate::project::Project;
use crate::{project::project_loaded, score::GameScore};

use super::GameViewport;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TextScale(1.0))
            .add_systems(Update, update_text_scale_system.run_if(project_loaded()))
            .add_systems(Startup, setup_combo_ui_system)
            .add_systems(Update, update_combo_system.run_if(project_loaded()))
            .add_systems(Update, hide_combo_below_3_system.run_if(project_loaded()))
            .add_systems(
                Update,
                update_combo_text_scale_system.run_if(project_loaded()),
            )
            .add_systems(Startup, spawn_score_ui_system)
            .add_systems(
                Update,
                update_score_text_scale_system.run_if(project_loaded()),
            )
            .add_systems(Update, update_score_system.run_if(project_loaded()))
            .add_systems(Startup, spawn_name_ui_system)
            .add_systems(
                Update,
                update_name_text_scale_system.run_if(project_loaded()),
            )
            .add_systems(Update, update_name_system.run_if(project_loaded()))
            .add_systems(Startup, spawn_level_ui_system)
            .add_systems(
                Update,
                update_level_text_scale_system.run_if(project_loaded()),
            )
            .add_systems(Update, update_level_system.run_if(project_loaded()));
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
                        },
                    ),
                    ..default()
                },
                ScoreText,
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
                left: Val::Px(8.0),
                bottom: Val::Px(8.0),
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
                bottom: Val::Px(8.0),
                right: Val::Px(8.0),
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
            ));
        });
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
    score: Res<GameScore>,
) {
    let mut score_text = score_text_query.single_mut();
    score_text.sections[0].value = score.score_text();
}

fn update_name_text_scale_system(
    mut name_text_query: Query<&mut Text, With<NameText>>,
    scale: Res<TextScale>,
) {
    let mut score_text = name_text_query.single_mut();
    score_text.sections[0].style.font_size = scale.0 * 1.32 * 0.5;
}

fn update_name_system(
    mut name_text_query: Query<&mut Text, With<NameText>>,
    project: Res<Project>,
) {
    let mut name_text = name_text_query.single_mut();
    name_text.sections[0].value.clone_from(&project.meta.name);
}

fn update_level_text_scale_system(
    mut name_text_query: Query<&mut Text, With<LevelText>>,
    scale: Res<TextScale>,
) {
    let mut score_text = name_text_query.single_mut();
    score_text.sections[0].style.font_size = scale.0 * 1.32 * 0.5;
}

fn update_level_system(
    mut name_text_query: Query<&mut Text, With<LevelText>>,
    project: Res<Project>,
) {
    let mut name_text = name_text_query.single_mut();
    name_text.sections[0].value.clone_from(&project.meta.level);
}
