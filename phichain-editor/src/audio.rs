use crate::settings::EditorSettings;
use crate::timing::{SeekToEvent, Timing};
use crate::utils::compat::ControlKeyExt;
use crate::{
    project::project_loaded,
    timing::{PauseEvent, Paused, ResumeEvent, SeekEvent},
};
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_persistent::prelude::*;
use phichain_game::audio::{AudioDuration, InstanceHandle};

/// Accumulated time delta (in seconds) for pending seek operations
///
/// When timeline_smooth_seeking is enabled, the delta is applied gradually;
/// when disabled, it's applied immediately.
#[derive(Resource)]
pub struct SeekDeltaTime(f32);

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SeekDeltaTime(0.0))
            .add_plugins(bevy_kira_audio::AudioPlugin)
            .add_systems(
                Update,
                (
                    handle_pause_system,
                    handle_resume_system,
                    handle_seek_system,
                    handle_seek_to_system,
                    update_seek_system,
                    update_volume_system,
                    update_playback_rate_system,
                    auto_pause_at_end_system,
                )
                    .run_if(
                        project_loaded()
                            .and(resource_exists::<InstanceHandle>)
                            .and(resource_exists::<AudioDuration>),
                    ),
            );
    }
}

// TODO: move this to separate plugin
/// When receiving [PauseEvent], pause the audio instance
fn handle_pause_system(
    handle: Res<InstanceHandle>,
    mut paused: ResMut<Paused>,
    mut game_paused: ResMut<phichain_game::Paused>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut events: EventReader<PauseEvent>,

    mut timing: ResMut<Timing>,
) {
    if let Some(instance) = audio_instances.get_mut(&handle.0) {
        for _ in events.read() {
            instance.pause(AudioTween::default());
            paused.0 = true;
            game_paused.0 = true;

            timing.pause();
        }
    }
}

/// Buffer from end of audio to prevent entering Stopped state
const END_BUFFER: f32 = 0.05;

/// The effective max time is slightly before the actual audio duration
/// to prevent the audio from reaching the end and entering the terminal
/// Stopped state in kira.
pub trait EffectiveMaxTime {
    fn effective_max_time(&self) -> f32;
}

impl EffectiveMaxTime for AudioDuration {
    fn effective_max_time(&self) -> f32 {
        self.0.as_secs_f32() - END_BUFFER
    }
}

// TODO: move this to separate plugin
/// When receiving [ResumeEvent], resume the audio instance
///
/// Resume is ignored if the current time is at or near the end of the audio
fn handle_resume_system(
    handle: Res<InstanceHandle>,
    mut paused: ResMut<Paused>,
    mut game_paused: ResMut<phichain_game::Paused>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut events: EventReader<ResumeEvent>,
    audio_duration: Res<AudioDuration>,

    mut timing: ResMut<Timing>,
) {
    if let Some(instance) = audio_instances.get_mut(&handle.0) {
        for _ in events.read() {
            // Don't resume if we're at or past the effective end
            if timing.now() >= audio_duration.effective_max_time() {
                continue;
            }

            instance.resume(AudioTween::default());
            paused.0 = false;
            game_paused.0 = false;
            timing.resume();
        }
    }
}

/// Apply accumulated [`SeekDeltaTime`] to [`Timing`] and the audio instance
///
/// All seek operations are clamped to valid range [0, duration - END_BUFFER]
fn update_seek_system(
    handle: Res<InstanceHandle>,
    paused: Res<Paused>,
    time: Res<Time>,
    settings: Res<Persistent<EditorSettings>>,
    audio_duration: Res<AudioDuration>,

    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut seek_delta_time: ResMut<SeekDeltaTime>,
    mut timing: ResMut<Timing>,
) {
    let max_time = audio_duration.effective_max_time();
    let delta = time.delta_secs();
    let now = timing.now();
    let seek_delta = seek_delta_time.0 * delta * 10.;
    let new_time = (now + seek_delta).clamp(0.0, max_time);
    timing.seek_to(new_time);
    seek_delta_time.0 -= seek_delta;

    // Directly seek the audio instance if not paused or if smooth seeking is disabled
    if (!paused.0 || !settings.general.timeline_smooth_seeking) && seek_delta_time.0.abs() > 0.0 {
        let final_time = (timing.now() + seek_delta_time.0).clamp(0.0, max_time);
        timing.seek_to(final_time);
        if let Some(instance) = audio_instances.get_mut(&handle.0) {
            instance.seek_to(final_time as f64);
        }
        seek_delta_time.0 = 0.0;
    }
}

// TODO: move this to separate plugin
/// Accumulates relative seek deltas to [`SeekDeltaTime`]
///
/// No immediate seeking occurs here - all timing changes are processed by [`update_seek_system`]
fn handle_seek_system(
    keyboard: Res<ButtonInput<KeyCode>>,

    mut events: EventReader<SeekEvent>,
    mut seek_target_time: ResMut<SeekDeltaTime>,
) {
    for event in events.read() {
        // holding Control will seek faster and holding Alt will seek slower
        let mut factor = 1.0;
        if keyboard.pressed(KeyCode::control()) {
            factor *= 2.0;
        }
        if keyboard.pressed(KeyCode::AltLeft) {
            factor /= 2.0;
        }
        seek_target_time.0 += event.0 * factor;
    }
}

// TODO: move this to separate plugin
/// Handles absolute timeline position changes with immediate synchronization.
///
/// This system:
/// 1. Immediately seeks both audio instance and editor timing to the target position
/// 2. Clears pending [`SeekDeltaTime`]
/// 3. Clamps seek target to valid range [0, duration - END_BUFFER]
fn handle_seek_to_system(
    handle: Res<InstanceHandle>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut events: EventReader<SeekToEvent>,
    mut seek_delta_time: ResMut<SeekDeltaTime>,
    audio_duration: Res<AudioDuration>,

    mut timing: ResMut<Timing>,
) {
    let max_time = audio_duration.effective_max_time();

    if let Some(instance) = audio_instances.get_mut(&handle.0) {
        for event in events.read() {
            let target = event.0.clamp(0.0, max_time);
            instance.seek_to(target.into());
            timing.seek_to(target);
            if seek_delta_time.0 > 0.0 {
                seek_delta_time.0 = 0.0;
            }
        }
    }
}

fn update_volume_system(
    handle: Res<InstanceHandle>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    settings: Res<Persistent<EditorSettings>>,
) {
    if let Some(instance) = audio_instances.get_mut(&handle.0) {
        instance.set_volume(
            Volume::Amplitude(settings.audio.music_volume as f64),
            AudioTween::default(),
        );
    }
}

fn update_playback_rate_system(
    handle: Res<InstanceHandle>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    settings: Res<Persistent<EditorSettings>>,
) {
    if let Some(instance) = audio_instances.get_mut(&handle.0) {
        instance.set_playback_rate(settings.audio.playback_rate as f64, AudioTween::default());
    }
}

/// Auto-pause when approaching end of audio to prevent entering Stopped state
fn auto_pause_at_end_system(
    timing: Res<Timing>,
    paused: Res<Paused>,
    audio_duration: Res<AudioDuration>,
    mut pause_events: EventWriter<PauseEvent>,
) {
    if !paused.0 && timing.now() >= audio_duration.effective_max_time() {
        pause_events.write_default();
    }
}
