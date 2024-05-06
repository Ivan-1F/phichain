mod assets;
mod audio;
mod chart;
mod hit_sound;
mod layer;
mod loader;
mod selection;
mod tab;
mod timing;

use crate::assets::{AssetsPlugin, ImageAssets};
use crate::audio::AudioPlugin;
use crate::chart::event::{LineEvent, LineEventKind};
use crate::chart::line::Line;
use crate::chart::note::TimelineNote;
use crate::chart::note::{Note, NoteKind};
use crate::layer::{EVENT_TIME_LINE_LAYER, GAME_LAYER, NOTE_TIME_LINE_LAYER};
use crate::loader::official::OfficialLoader;
use crate::loader::Loader;
use crate::tab::game::GameCamera;
use crate::tab::game::GameTabPlugin;
use crate::tab::game::GameViewport;
use crate::tab::timeline::{
    EventTimelineCamera, NoteTimelineCamera, TimelineTabPlugin, TimelineViewport,
};
use crate::timing::{ChartTime, TimingPlugin};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::sprite::Anchor;
use bevy::{prelude::*, render::view::RenderLayers};
use bevy_egui::egui::{Color32, Frame};
use bevy_egui::{EguiContext, EguiPlugin};
use bevy_mod_picking::prelude::*;
use chart::beat::Beat;
use chart::line::{LineOpacity, LinePosition, LineRotation};
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use fraction::prelude::*;
use selection::SelectedLine;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(UiState::new())
        .add_plugins(DefaultPlugins)
        .add_plugins(TimingPlugin)
        .add_plugins(AudioPlugin)
        .add_plugins(GameTabPlugin)
        .add_plugins(TimelineTabPlugin)
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(AssetsPlugin)
        .add_systems(Startup, setup_plugin)
        .add_systems(Startup, setup_chart_plugin)
        .add_systems(Update, zoom_scale)
        .add_systems(Update, ui_system)
        .add_systems(
            Update,
            (
                update_note_scale_system,
                update_note_system,
                update_note_y_system,
                update_note_texture_system,
            )
                .chain(),
        )
        // .add_systems(Update, update_judgeline_system)
        .add_systems(Update, (compute_line_system, update_line_system))
        .add_systems(
            Update,
            (update_line_texture_system, update_note_texture_system),
        )
        .add_systems(Update, calculate_speed_events_system)
        .add_systems(Update, update_timeline_note_system)
        .add_systems(Update, update_timeline_event_system)
        .add_systems(Update, hide_unselected_line_timeline_items_system)
        .run();
}

fn hide_unselected_line_timeline_items_system(
    selected_line: Res<SelectedLine>,
    mut event_query: Query<(&LineEvent, &mut Visibility), Without<TimelineNote>>,
    mut timeline_note_query: Query<(&TimelineNote, &mut Visibility), Without<LineEvent>>,
    note_query: Query<&Parent, With<Note>>,
) {
    for (event, mut visibility) in &mut event_query {
        *visibility = if event.line_id == selected_line.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (note, mut visibility) in &mut timeline_note_query {
        let parent: &Parent = note_query.get(note.0).unwrap();
        *visibility = if parent.get() == selected_line.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

struct TabViewer<'a> {
    world: &'a mut World,
}

#[derive(Debug)]
enum EditorTab {
    Game,
    Timeline,
    Inspector,
}

#[derive(Resource)]
struct UiState {
    state: DockState<EditorTab>,
}

impl UiState {
    fn new() -> Self {
        let mut state = DockState::new(vec![EditorTab::Game]);
        let tree = state.main_surface_mut();
        let [game, _timeline] =
            tree.split_left(NodeIndex::root(), 1.0 / 2.0, vec![EditorTab::Timeline]);

        tree.split_below(game, 1.0 / 2.0, vec![EditorTab::Inspector]);

        Self { state }
    }

    fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        let mut tab_viewer = TabViewer { world };

        DockArea::new(&mut self.state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);
    }
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = EditorTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        format!("{tab:?}").into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            EditorTab::Game => {
                let mut game_viewport = self.world.resource_mut::<GameViewport>();
                let clip_rect = ui.clip_rect();
                game_viewport.0 = Rect::from_corners(
                    Vec2 {
                        x: clip_rect.min.x,
                        y: clip_rect.min.y,
                    },
                    Vec2 {
                        x: clip_rect.max.x,
                        y: clip_rect.max.y,
                    },
                );
            }
            EditorTab::Timeline => {
                let mut timeline_viewport = self.world.resource_mut::<TimelineViewport>();
                let clip_rect = ui.clip_rect();
                timeline_viewport.0 = Rect::from_corners(
                    Vec2 {
                        x: clip_rect.min.x,
                        y: clip_rect.min.y,
                    },
                    Vec2 {
                        x: clip_rect.max.x,
                        y: clip_rect.max.y,
                    },
                );
            },
            _ => {}
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        self.allowed_in_windows(tab)
    }

    fn allowed_in_windows(&self, tab: &mut Self::Tab) -> bool {
        !matches!(tab, EditorTab::Game | EditorTab::Timeline)
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, EditorTab::Game | EditorTab::Timeline)
    }
}

fn ui_system(world: &mut World) {
    let egui_context = world.query::<&mut EguiContext>().single_mut(world);
    let mut egui_context = egui_context.clone();
    let ctx = egui_context.get_mut();

    let diagnostics = world.resource::<DiagnosticsStore>();
    let mut fps = 0.0;
    if let Some(value) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
    {
        fps = value;
    }

    egui::TopBottomPanel::top("phichain.MenuBar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Quit").clicked() {
                    std::process::exit(0);
                }
            });
        });
    });

    egui::TopBottomPanel::bottom("phichain.StatusBar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("PhiChain v0.1.0");

            ui.label(format!("FPS: {:.2}", fps));
        });
    });

    egui::CentralPanel::default()
        .frame(Frame {
            fill: Color32::TRANSPARENT,
            ..default()
        })
        .show(ctx, |_ui| {
            world.resource_scope::<UiState, _>(|world, mut ui_state| {
                ui_state.ui(world, &mut ctx.clone());
            });
        });
}

fn setup_plugin(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 0,
                ..default()
            },
            ..default()
        },
        RenderLayers::layer(GAME_LAYER),
        GameCamera,
    ));
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 1,
                ..default()
            },
            ..default()
        },
        RenderLayers::layer(NOTE_TIME_LINE_LAYER),
        NoteTimelineCamera,
    ));
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 2,
                ..default()
            },
            ..default()
        },
        RenderLayers::layer(EVENT_TIME_LINE_LAYER),
        EventTimelineCamera,
    ));
}

fn update_timeline_note_system(
    mut query: Query<(
        &mut Transform,
        &mut Sprite,
        &mut Handle<Image>,
        &TimelineNote,
    )>,
    note_query: Query<(&Note, &Parent)>,
    assets: Res<ImageAssets>,
    timeline_viewport: Res<TimelineViewport>,
    time: Res<ChartTime>,
    selected_line: Res<SelectedLine>,
) {
    let note_timeline_viewport = timeline_viewport.note_timeline_viewport();
    for (mut transform, mut sprite, mut image, timeline_note) in &mut query {
        let (note, parent) = note_query.get(timeline_note.0).unwrap();
        if parent.get() != selected_line.0 {
            continue;
        }
        match note.kind {
            NoteKind::Tap => *image = assets.tap.clone(),
            NoteKind::Drag => *image = assets.drag.clone(),
            NoteKind::Hold { hold_beat: _ } => *image = assets.hold.clone(),
            NoteKind::Flick => *image = assets.flick.clone(),
        }
        transform.scale = Vec3::splat(note_timeline_viewport.width() / 8000.0);
        transform.translation.y = note_timeline_viewport.height() / 2.0
            - (note_timeline_viewport.height()
                - (note.beat.value() * (60.0 / 174.0) - time.0) * 400.0 * 2.0);
        transform.translation.x = note.x * note_timeline_viewport.width();
        match note.kind {
            NoteKind::Hold { hold_beat } => {
                sprite.anchor = Anchor::BottomCenter;
                transform.scale.y = (hold_beat.value() * (60.0 / 174.0) * 400.0 * 2.0) / 1900.0;
            }
            _ => sprite.anchor = Anchor::Center,
        }
    }
}

fn update_timeline_event_system(
    mut query: Query<(&mut Transform, &mut Sprite, &mut Handle<Image>, &LineEvent)>,
    assets: Res<ImageAssets>,
    timeline_viewport: Res<TimelineViewport>,
    time: Res<ChartTime>,
    selected_line: Res<SelectedLine>,
) {
    let event_timeline_viewport = timeline_viewport.event_timeline_viewport();
    for (mut transform, mut sprite, mut image, event) in &mut query {
        if event.line_id != selected_line.0 {
            continue;
        }
        *image = assets.hold.clone();
        transform.scale = Vec3::splat(event_timeline_viewport.width() / 8000.0);
        transform.translation.y = event_timeline_viewport.height() / 2.0
            - (event_timeline_viewport.height()
                - (event.start_beat.value() * (60.0 / 174.0) - time.0) * 400.0 * 2.0);
        match event.kind {
            LineEventKind::X => {
                transform.translation.x = -event_timeline_viewport.width() / 2.0 / 5.0 * (2.0 * 2.0)
            }
            LineEventKind::Y => {
                transform.translation.x = -event_timeline_viewport.width() / 2.0 / 5.0 * (1.0 * 2.0)
            }
            LineEventKind::Rotation => transform.translation.x = 0.0,
            LineEventKind::Opacity => {
                transform.translation.x = event_timeline_viewport.width() / 2.0 / 5.0 * (1.0 * 2.0)
            }
            LineEventKind::Speed => {
                transform.translation.x = event_timeline_viewport.width() / 2.0 / 5.0 * (2.0 * 2.0)
            }
        }
        sprite.anchor = Anchor::BottomCenter;
        transform.scale.y =
            ((event.end_beat.value() - event.start_beat.value()) * (60.0 / 174.0) * 400.0 * 2.0)
                / 1900.0;
    }
}

fn zoom_scale(
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

/// Load a chart in official JSON format into the world
fn setup_chart_plugin(commands: Commands) {
    OfficialLoader::load(
        std::fs::File::open("Chart_AT.json").expect("Failed to open chart"),
        commands,
    );
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
    mut query: Query<(&mut Transform, &mut Sprite, &Note)>,
    game_viewport: Res<GameViewport>,
    time: Res<ChartTime>,
) {
    let beat = time.0 / (60.0 / 174.0);
    for (mut transform, mut sprite, note) in &mut query {
        transform.translation.x =
            note.x * game_viewport.0.width() / (game_viewport.0.width() * 3.0 / 1920.0);
        let hold_beat = if let NoteKind::Hold { hold_beat } = note.kind {
            hold_beat.value()
        } else {
            0.0
        };
        sprite.color = Color::WHITE.with_a(if note.beat.value() + hold_beat < beat {
            0.0
        } else {
            1.0
        })
    }
}

fn compute_line_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    event_query: Query<&LineEvent>,
    mut line_query: Query<
        (
            &mut LinePosition,
            &mut LineRotation,
            &mut LineOpacity,
            Entity,
        ),
        With<Line>,
    >,
    time: Res<ChartTime>,
) {
    let beat = time.0 / (60.0 / 174.0);
    for (mut position, mut rotation, mut opacity, entity) in &mut line_query {
        let mut events: Vec<_> = event_query.iter().filter(|e| e.line_id == entity).collect();
        events.sort_by_key(|e| <Beat as Into<Fraction>>::into(e.start_beat));
        for event in events {
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
                            opacity.0 = value;
                        }
                    }
                    LineEventKind::Speed => {}
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
        transform.translation.x = position.0.x * game_viewport.0.width();
        transform.translation.y = position.0.y * game_viewport.0.height();
        transform.rotation = Quat::from_rotation_z(rotation.0);
        sprite.color = Color::rgba(1.0, 1.0, 1.0, opacity.0);
    }
}

fn update_note_y_system(
    query: Query<(&Children, Entity), With<Line>>,
    game_viewport: Res<GameViewport>,
    speed_event_query: Query<(&SpeedEvent, &LineEvent)>,
    mut note_query: Query<(&mut Transform, &mut Sprite, &Note)>,
    time: Res<ChartTime>,
) {
    let all_speed_events: Vec<(&SpeedEvent, &LineEvent)> = speed_event_query.iter().collect();
    for (children, entity) in &query {
        let mut speed_events: Vec<&SpeedEvent> = all_speed_events
            .iter()
            .filter(|(_, e)| e.line_id == entity)
            .map(|(s, _)| *s)
            .collect();
        speed_events
            .sort_by(|a, b| Fraction::from(a.start_time).cmp(&Fraction::from(b.start_time)));

        let distance = |time| {
            distance_at(&speed_events, time) * (game_viewport.0.height() * (120.0 / 900.0))
                / (game_viewport.0.width() * 3.0 / 1920.0)
        };
        let current_distance = distance(time.0);
        for child in children {
            if let Ok((mut transform, mut sprite, note)) = note_query.get_mut(*child) {
                let mut y = distance(note.beat.value() * (60.0 / 174.0)) - current_distance;
                match note.kind {
                    NoteKind::Hold { hold_beat } => {
                        y = y.max(0.0);
                        let height = distance(
                            note.beat.value() * (60.0 / 174.0) + hold_beat.value() * (60.0 / 174.0),
                        ) - current_distance
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
    mut query: Query<(&mut Handle<Image>, &Note)>,
    assets: Res<ImageAssets>,
) {
    for (mut image, note) in &mut query {
        match note.kind {
            NoteKind::Tap => *image = assets.tap.clone(),
            NoteKind::Drag => *image = assets.drag.clone(),
            NoteKind::Hold { hold_beat: _ } => *image = assets.hold.clone(),
            NoteKind::Flick => *image = assets.flick.clone(),
        }
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
        return Self {
            start_time,
            end_time,
            start_value,
            end_value,
        };
    }
}

fn calculate_speed_events_system(mut commands: Commands, query: Query<(&LineEvent, Entity)>) {
    for (event, entity) in &query {
        match event.kind {
            LineEventKind::Speed => {
                commands.entity(entity).insert(SpeedEvent::new(
                    event.start_beat.value() * (60.0 / 174.0),
                    event.end_beat.value() * (60.0 / 174.0),
                    event.start,
                    event.end,
                ));
            }
            _ => {}
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
            // t = event.start_time;
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
