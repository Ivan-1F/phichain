pub mod core;
pub mod hit_effect;
pub mod illustration;
pub mod ui;

use crate::audio::AudioDuration;
use bevy::{prelude::*, render::camera::Viewport};
use egui::Ui;
use phichain_chart::bpm_list::BpmList;

use crate::project::project_loaded;
use crate::tab::game::hit_effect::HitEffectPlugin;
use crate::timing::{ChartTime, SeekToEvent};

use self::{core::CoreGamePlugin, illustration::IllustrationPlugin, ui::GameUiPlugin};

pub fn game_tab(
    In(ui): In<&mut Ui>,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
    duration: Res<AudioDuration>,
    mut events: EventWriter<SeekToEvent>,
) {
    let seconds = time.0;
    let mut second_binding = seconds;
    let beats = bpm_list.beat_at(seconds).value();
    let mut beat_binding = beats;

    ui.horizontal(|ui| {
        ui.add(
            egui::Slider::new(&mut second_binding, 0.0..=duration.0.as_secs_f32())
                .custom_formatter(|x, _| format!("{:.2}", x))
                .drag_value_speed(0.05),
        );
        let max_beat = bpm_list.beat_at(duration.0.as_secs_f32());
        ui.add(
            egui::DragValue::new(&mut beat_binding)
                .speed(0.05)
                .custom_formatter(|x, _| format!("{:.2}", x))
                .clamp_range(0.0..=max_beat.value()),
        );
    });

    if second_binding != seconds {
        events.send(SeekToEvent(second_binding));
    }

    if beat_binding != beats {
        events.send(SeekToEvent(bpm_list.time_at(beat_binding.into())));
    }
}

pub struct GameTabPlugin;

impl Plugin for GameTabPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameViewport(Rect::from_corners(Vec2::ZERO, Vec2::ZERO)))
            .add_systems(
                PostUpdate,
                update_game_camera_viewport.run_if(project_loaded()),
            )
            .add_plugins(GameUiPlugin)
            .add_plugins(IllustrationPlugin)
            .add_plugins(CoreGamePlugin)
            .add_plugins(HitEffectPlugin);
    }
}

#[derive(Resource, Debug)]
pub struct GameViewport(pub Rect);

#[derive(Component)]
pub struct GameCamera;

fn update_game_camera_viewport(
    mut query: Query<&mut Camera, With<GameCamera>>,
    window_query: Query<&Window>,
    egui_settings: Res<bevy_egui::EguiSettings>,
    game_viewport: Res<GameViewport>,
) {
    let mut game_camera = query.single_mut();
    let Ok(window) = window_query.get_single() else {
        return;
    };

    let scale_factor = window.scale_factor() * egui_settings.scale_factor;
    let viewport_pos = game_viewport.0.min * scale_factor;
    let viewport_size = game_viewport.0.size() * scale_factor;

    if viewport_pos.x < 0.0
        || viewport_pos.y < 0.0
        || viewport_size.x <= 0.0
        || viewport_size.y <= 0.0
        || viewport_pos.x + viewport_size.x > window.width() * scale_factor
        || viewport_pos.y + viewport_size.y > window.height() * scale_factor
    {
        return;
    }

    game_camera.viewport = Some(Viewport {
        physical_position: viewport_pos.as_uvec2(),
        physical_size: viewport_size.as_uvec2(),
        depth: 0.0..1.0,
    });
}
