use super::{GameConfig, GameSet, GameViewport};
use crate::audio::AudioDuration;
use crate::score::GameScore;
use crate::utils::text_utils::{split_by_script, Script};
use crate::{ChartTime, PauseToggleRequest, SeekRequest};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

const CJK_FONT: &str = "font/MiSans-Regular.ttf";
const ASCII_FONT: &str = "font/phigros.ttf";

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BaseTextScale(1.0))
            .add_systems(
                Update,
                (update_base_text_scale_system, update_text_scale_system)
                    .chain()
                    .in_set(GameSet),
            )
            .add_systems(Update, update_ui_text_margin_system)
            // combo
            .add_systems(Startup, setup_combo_ui_system)
            .add_systems(Update, update_combo_system.in_set(GameSet))
            .add_systems(Update, hide_combo_below_3_system.in_set(GameSet))
            // score
            .add_systems(Startup, spawn_score_ui_system)
            .add_systems(Update, update_score_system.in_set(GameSet))
            // name
            .add_systems(Startup, spawn_name_ui_system)
            .add_systems(
                Update,
                update_name_system
                    .in_set(GameSet)
                    .run_if(resource_exists_and_changed::<GameConfig>)
                    .before(update_text_scale_system), // make sure update_name_system will not override font_size
            )
            // level
            .add_systems(Startup, spawn_level_ui_system)
            .add_systems(
                Update,
                update_level_system
                    .in_set(GameSet)
                    .run_if(resource_exists_and_changed::<GameConfig>)
                    .before(update_text_scale_system), // make sure update_level_system will not override font_size
            )
            // progress bar
            .add_systems(Startup, spawn_progress_bar_system)
            .add_systems(
                Update,
                update_progress_bar_system
                    .in_set(GameSet)
                    .run_if(resource_exists::<AudioDuration>),
            )
            // pause button
            .add_systems(Startup, spawn_pause_button_system)
            .add_systems(
                Update,
                update_pause_button_size_system
                    .after(update_base_text_scale_system)
                    .in_set(GameSet),
            );
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

#[derive(Component, Debug)]
struct ApplyMargin {
    left: bool,
    right: bool,
    top: bool,
    bottom: bool,
}

impl ApplyMargin {
    fn all() -> Self {
        Self {
            left: true,
            right: true,
            top: true,
            bottom: true,
        }
    }

    fn none() -> Self {
        Self {
            left: false,
            right: false,
            top: false,
            bottom: false,
        }
    }
}

fn update_ui_text_margin_system(
    mut query: Query<(&mut Node, &ApplyMargin)>,
    scale: Res<BaseTextScale>,
) {
    for (mut node, sides) in &mut query {
        let value = Val::Px(scale.0 * 0.5);
        let mut rect = UiRect::ZERO;
        if sides.left {
            rect.left = value;
        }
        if sides.right {
            rect.right = value;
        }
        if sides.top {
            rect.top = value;
        }
        if sides.bottom {
            rect.bottom = value;
        }
        node.margin = rect;
    }
}

fn setup_combo_ui_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                top: Val::Px(0.0),
                ..default()
            },
            ApplyMargin {
                left: false,
                right: false,
                top: true,
                bottom: false,
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        align_self: AlignSelf::Center,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Combo,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("COMBO"), // this will be replaced every frame at update_combo_system
                        TextFont {
                            font: asset_server.load("font/phigros.ttf"),
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor::WHITE,
                        ComboText,
                        TextScale(1.0),
                    ));

                    parent.spawn((
                        Text::new("COMBO"),
                        TextFont {
                            font: asset_server.load("font/phigros.ttf"),
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor::WHITE,
                        ComboIndicator,
                        TextScale(0.4),
                        ApplyMargin::none(),
                    ));
                });
        });
}

/// Marker component to represent the score text
#[derive(Component)]
struct ScoreText;

fn spawn_score_ui_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                right: Val::Px(0.0),
                ..default()
            },
            ApplyMargin::all(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("0000000"),
                TextFont {
                    font: asset_server.load("font/phigros.ttf"),
                    font_size: 10.0,
                    ..default()
                },
                TextColor::WHITE,
                ScoreText,
                TextScale(0.8),
            ));
        });
}

/// Marker component to represent the name text
#[derive(Component)]
struct NameText;

fn spawn_name_ui_system(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(0.0),
                ..default()
            },
            ApplyMargin::all(),
        ))
        .with_children(|parent| {
            parent.spawn((Text::default(), NameText));
        });
}

/// Marker component to represent the level text
#[derive(Component)]
struct LevelText;

fn spawn_level_ui_system(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                right: Val::Px(0.0),
                ..default()
            },
            ApplyMargin::all(),
        ))
        .with_children(|parent| {
            parent.spawn((Text::default(), LevelText));
        });
}

fn update_text_scale_system(
    scale: Res<BaseTextScale>,
    mut query: Query<(&mut TextFont, &TextScale)>,
) {
    for (mut text, text_scale) in &mut query {
        text.font_size = scale.0 * 1.32 * text_scale.0;
    }
}

fn update_combo_system(
    mut text_query: Query<&mut Text, With<ComboText>>,
    score: Res<GameScore>,
) -> Result {
    let mut combo_text = text_query.single_mut()?;
    **combo_text = score.combo().to_string();

    Ok(())
}

fn hide_combo_below_3_system(
    mut combo_query: Query<&mut Visibility, With<Combo>>,
    score: Res<GameScore>,
) -> Result {
    let mut visibility = combo_query.single_mut()?;
    *visibility = if score.combo() >= 3 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    Ok(())
}

fn update_score_system(
    mut score_text_query: Query<&mut Text, With<ScoreText>>,
    score: Res<GameScore>,
) -> Result {
    let mut score_text = score_text_query.single_mut()?;
    **score_text = score.score_text();

    Ok(())
}

fn update_name_system(
    mut commands: Commands,
    mut name_text_query: Query<Entity, With<NameText>>,
    config: Res<GameConfig>,
    asset_server: Res<AssetServer>,
) -> Result {
    let container = name_text_query.single_mut()?;

    commands.entity(container).despawn_related::<Children>();
    commands.entity(container).with_children(|parent| {
        for (content, script) in split_by_script(&config.name.replace(' ', "\u{00A0}")) {
            parent.spawn((
                Text::new(content),
                TextFont {
                    font: asset_server.load(match script {
                        Script::Ascii => ASCII_FONT,
                        Script::Cjk => CJK_FONT,
                    }),
                    font_size: 10.0,
                    ..default()
                },
                TextColor::WHITE,
                TextScale(0.5),
            ));
        }
    });

    Ok(())
}

/// Marker component for the progress bar root (also acts as the hit area).
#[derive(Component)]
struct ProgressBar;

/// Marker component for the progress bar filled region
#[derive(Component)]
struct ProgressBarFill;

/// Marker component for the progress bar leading edge indicator
#[derive(Component)]
struct ProgressBarMarker;

const PROGRESS_BAR_HIT_AREA_PERCENT: f32 = 2.0;
const PROGRESS_BAR_VISUAL_HEIGHT_PERCENT: f32 = 50.0;
const PROGRESS_BAR_MARKER_WIDTH_PERCENT: f32 = 0.3;

fn spawn_progress_bar_system(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(PROGRESS_BAR_HIT_AREA_PERCENT),
                ..default()
            },
            RelativeCursorPosition::default(),
            ProgressBar,
        ))
        .observe(on_progress_bar_press)
        .observe(on_progress_bar_drag)
        .with_children(|parent| {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(0.0),
                    height: Val::Percent(PROGRESS_BAR_VISUAL_HEIGHT_PERCENT),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.6)),
                ProgressBarFill,
                Pickable::IGNORE,
            ));
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Percent(-PROGRESS_BAR_MARKER_WIDTH_PERCENT / 2.0),
                    width: Val::Percent(PROGRESS_BAR_MARKER_WIDTH_PERCENT),
                    height: Val::Percent(PROGRESS_BAR_VISUAL_HEIGHT_PERCENT),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                ProgressBarMarker,
                Pickable::IGNORE,
            ));
        });
}

fn update_progress_bar_system(
    time: Res<ChartTime>,
    audio_duration: Res<AudioDuration>,
    mut fill_query: Query<&mut Node, (With<ProgressBarFill>, Without<ProgressBarMarker>)>,
    mut marker_query: Query<&mut Node, (With<ProgressBarMarker>, Without<ProgressBarFill>)>,
) -> Result {
    let total = audio_duration.0.as_secs_f32();
    let progress = if total > 0.0 {
        (time.0 / total).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let mut fill = fill_query.single_mut()?;
    fill.width = Val::Percent(progress * 100.0);

    let mut marker = marker_query.single_mut()?;
    marker.left = Val::Percent(progress * 100.0 - PROGRESS_BAR_MARKER_WIDTH_PERCENT / 2.0);

    Ok(())
}

fn on_progress_bar_press(
    event: On<Pointer<Press>>,
    query: Query<&RelativeCursorPosition, With<ProgressBar>>,
    audio_duration: Option<Res<AudioDuration>>,
    mut seek: MessageWriter<SeekRequest>,
) {
    emit_progress_bar_seek(
        event.entity,
        event.button,
        &query,
        audio_duration.as_deref(),
        &mut seek,
    );
}

fn on_progress_bar_drag(
    event: On<Pointer<Drag>>,
    query: Query<&RelativeCursorPosition, With<ProgressBar>>,
    audio_duration: Option<Res<AudioDuration>>,
    mut seek: MessageWriter<SeekRequest>,
) {
    emit_progress_bar_seek(
        event.entity,
        event.button,
        &query,
        audio_duration.as_deref(),
        &mut seek,
    );
}

/// Marker component for the pause button root.
#[derive(Component)]
struct PauseButton;

// Button geometry is driven by `BaseTextScale` (viewport's short side), so the button
// scales with the same metric as the text HUD and stays visually consistent across
// game viewport sizes.
const PAUSE_HIT_AREA_TOP_FACTOR: f32 = 0.66;
const PAUSE_HIT_AREA_LEFT_FACTOR: f32 = 0.5;

const PAUSE_BAR_WIDTH_PERCENT: f32 = 25.0;
const PAUSE_BAR_HEIGHT_PERCENT: f32 = 80.0;
const PAUSE_BAR_TOP_PERCENT: f32 = 0.0;
const PAUSE_BAR_LEFT_PERCENTS: [f32; 2] = [12.5, 62.5];

fn spawn_pause_button_system(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::NONE),
            // Lift above other in-game UI roots so picking can reach the button; otherwise
            // the combo/score root nodes (stack_index > 0) sit on top and block clicks.
            GlobalZIndex(10),
            PauseButton,
        ))
        .observe(on_pause_button_press)
        .with_children(|parent| {
            for left in PAUSE_BAR_LEFT_PERCENTS {
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Percent(PAUSE_BAR_TOP_PERCENT),
                        left: Val::Percent(left),
                        width: Val::Percent(PAUSE_BAR_WIDTH_PERCENT),
                        height: Val::Percent(PAUSE_BAR_HEIGHT_PERCENT),
                        ..default()
                    },
                    BackgroundColor(Color::WHITE),
                    Pickable::IGNORE,
                ));
            }
        });
}

fn update_pause_button_size_system(
    scale: Res<BaseTextScale>,
    mut query: Query<&mut Node, With<PauseButton>>,
) -> Result {
    let s = scale.0;
    let mut node = query.single_mut()?;
    node.width = Val::Px(s);
    node.height = Val::Px(s);
    node.top = Val::Px(s * PAUSE_HIT_AREA_TOP_FACTOR);
    node.left = Val::Px(s * PAUSE_HIT_AREA_LEFT_FACTOR);
    Ok(())
}

fn on_pause_button_press(event: On<Pointer<Press>>, mut toggle: MessageWriter<PauseToggleRequest>) {
    if event.button != PointerButton::Primary {
        return;
    }
    toggle.write(PauseToggleRequest);
}

fn emit_progress_bar_seek(
    entity: Entity,
    button: PointerButton,
    query: &Query<&RelativeCursorPosition, With<ProgressBar>>,
    audio_duration: Option<&AudioDuration>,
    seek: &mut MessageWriter<SeekRequest>,
) {
    if button != PointerButton::Primary {
        return;
    }
    let Some(audio_duration) = audio_duration else {
        return;
    };
    let Ok(rcp) = query.get(entity) else {
        return;
    };
    let Some(normalized) = rcp.normalized else {
        return;
    };
    // RelativeCursorPosition's normalized is in [-0.5, 0.5]; shift to [0, 1] along the bar.
    let progress = (normalized.x + 0.5).clamp(0.0, 1.0);
    seek.write(SeekRequest(progress * audio_duration.0.as_secs_f32()));
}

fn update_level_system(
    mut commands: Commands,
    mut name_text_query: Query<Entity, With<LevelText>>,
    config: Res<GameConfig>,
    asset_server: Res<AssetServer>,
) -> Result {
    let container = name_text_query.single_mut()?;

    commands.entity(container).despawn_related::<Children>();
    commands.entity(container).with_children(|parent| {
        for (content, script) in split_by_script(&config.level.replace(' ', "\u{00A0}")) {
            parent.spawn((
                Text::new(content),
                TextFont {
                    font: asset_server.load(match script {
                        Script::Ascii => ASCII_FONT,
                        Script::Cjk => CJK_FONT,
                    }),
                    font_size: 10.0,
                    ..default()
                },
                TextColor::WHITE,
                TextScale(0.5),
            ));
        }
    });

    Ok(())
}
