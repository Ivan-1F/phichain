mod data;

use crate::timeline::TimelineContext;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy::tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task};
use bevy_kira_audio::prelude::StaticSoundData;
use egui::{pos2, Color32, Mesh, Rect, Ui};
use phichain_chart::offset::Offset;

use data::Waveform;

const WAVEFORM_OPACITY: f32 = 0.4;

#[derive(Resource)]
pub struct PendingWaveform(Task<Waveform>);

pub fn start_generation(world: &mut World, sound: StaticSoundData) {
    clear(world);
    world.insert_resource(PendingWaveform(
        AsyncComputeTaskPool::get().spawn(async move { Waveform::new(sound) }),
    ));
}

pub fn clear(world: &mut World) {
    world.remove_resource::<Waveform>();
    // Dropping the task also prevents an old project's result from being installed.
    world.remove_resource::<PendingWaveform>();
}

pub fn poll_waveform_system(mut commands: Commands, pending: Option<ResMut<PendingWaveform>>) {
    let Some(mut pending) = pending else {
        return;
    };
    if let Some(waveform) = block_on(future::poll_once(&mut pending.0)) {
        commands.remove_resource::<PendingWaveform>();
        commands.insert_resource(waveform);
    }
}

/// Draw one shared waveform across the timeline, beneath its grid and content.
pub fn draw(ui: &Ui, world: &mut World, rect: Rect) {
    let mut state =
        SystemState::<(TimelineContext, Option<Res<Waveform>>, Res<Offset>)>::new(world);
    let (ctx, waveform, offset) = state.get(world);
    if !ctx.settings.show_waveform || !rect.is_positive() {
        return;
    }
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));

    let center = rect.center().x;
    let half_width = (rect.width() * 0.5 - 4.0).max(0.0);

    if let Some(waveform) = waveform {
        let color = ui.visuals().text_color().gamma_multiply(WAVEFORM_OPACITY);
        let mut mesh = Mesh::default();
        // Query the full interval covered by each physical pixel, so zooming out
        // preserves short transients instead of skipping samples between rows.
        let rows = (rect.height() * ui.ctx().pixels_per_point()).ceil() as usize;
        let row_height = rect.height() / rows.max(1) as f32;
        for row in 0..rows {
            let top = rect.top() + row as f32 * row_height;
            let bottom = (top + row_height).min(rect.bottom());
            // Chart time excludes Offset, whereas the samples use audio time.
            let audio_time = |y| f64::from(ctx.y_to_time(y)) + f64::from(offset.0) / 1000.0;
            let Some(peak) = waveform.peaks_between(audio_time(bottom), audio_time(top)) else {
                continue;
            };
            if peak.min == peak.max {
                continue;
            }
            mesh.add_colored_rect(
                Rect::from_min_max(
                    pos2(center + peak.min.clamp(-1.0, 1.0) * half_width, top),
                    pos2(center + peak.max.clamp(-1.0, 1.0) * half_width, bottom),
                ),
                color,
            );
        }
        painter.add(mesh);
    } else {
        let galley = painter.layout(
            t!("tab.timeline_setting.waveform_loading").into_owned(),
            egui::FontId::proportional(11.0),
            ui.visuals().weak_text_color(),
            (rect.width() - 8.0).max(1.0),
        );
        painter.galley(rect.min + egui::vec2(4.0, 4.0), galley, Color32::WHITE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tab::timeline::TimelineViewport;
    use crate::timeline::settings::TimelineSettings;
    use crate::timing::ChartTime;
    use bevy_kira_audio::prelude::Frame;
    use phichain_chart::bpm_list::BpmList;
    use phichain_game::audio::AudioDuration;

    fn impulse() -> StaticSoundData {
        let mut frames = vec![Frame::from_mono(0.0); 4096];
        frames[2048] = Frame::from_mono(1.0);
        StaticSoundData {
            sample_rate: 1024,
            frames: frames.into(),
            settings: Default::default(),
            slice: None,
        }
    }

    #[test]
    fn rendered_impulse_aligns_with_chart_time_at_all_zoom_limits_and_offset_signs() {
        let rect = Rect::from_min_max(pos2(10.0, 30.0), pos2(500.0, 630.0));
        for zoom in [0.1, 1.0, 5.0] {
            for offset_ms in [-500.0, 0.0, 500.0] {
                let mut world = World::new();
                world.insert_resource(TimelineSettings {
                    zoom,
                    ..Default::default()
                });
                world.insert_resource(TimelineViewport(bevy::math::Rect::new(
                    10.0, 30.0, 800.0, 630.0,
                )));
                world.insert_resource(BpmList::default());
                world.insert_resource(ChartTime(1.95 - offset_ms / 1000.0));
                world.insert_resource(Offset(offset_ms));
                world.insert_resource(AudioDuration(std::time::Duration::from_secs(4)));
                world.insert_resource(Waveform::new(impulse()));
                let mut state = SystemState::<TimelineContext>::new(&mut world);
                let expected_y = state.get(&world).time_to_y(2.0 - offset_ms / 1000.0);

                let context = egui::Context::default();
                let output = context.run(egui::RawInput::default(), |context| {
                    egui::CentralPanel::default().show(context, |ui| draw(ui, &mut world, rect));
                });
                let vertices: Vec<_> = output
                    .shapes
                    .iter()
                    .filter_map(|shape| {
                        if let egui::Shape::Mesh(mesh) = &shape.shape {
                            Some(&mesh.vertices)
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .collect();
                assert!(!vertices.is_empty());
                let top = vertices
                    .iter()
                    .map(|v| v.pos.y)
                    .fold(f32::INFINITY, f32::min);
                let bottom = vertices
                    .iter()
                    .map(|v| v.pos.y)
                    .fold(f32::NEG_INFINITY, f32::max);
                // One sample's duration plus raster rounding; no analysis-window delay.
                assert!(
                    (top - expected_y).abs() <= crate::constants::BASE_ZOOM * zoom / 1024.0 + 1.1
                );
                assert!((bottom - expected_y).abs() <= 1.1);
                assert!(vertices.iter().all(|v| rect.contains(v.pos)));
            }
        }
    }

    #[test]
    fn closing_a_project_removes_ready_and_pending_waveforms() {
        let pool = bevy::tasks::TaskPool::new();
        let mut app = App::new();
        app.add_systems(Update, poll_waveform_system);
        app.insert_resource(Waveform::new(impulse()));
        app.insert_resource(PendingWaveform(pool.spawn(std::future::pending())));

        clear(app.world_mut());
        app.update();

        assert!(!app.world().contains_resource::<Waveform>());
        assert!(!app.world().contains_resource::<PendingWaveform>());
    }
}
