use std::time::Duration;
use std::{io::Cursor, path::PathBuf};

use crate::settings::EditorSettings;
use crate::spectrogram::Spectrogram;
use crate::timing::{SeekToEvent, Timing};
use crate::utils::compat::ControlKeyExt;
use crate::{
    project::project_loaded,
    spectrogram,
    timing::{PauseEvent, Paused, ResumeEvent, SeekEvent},
};
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_persistent::prelude::*;
use thiserror::Error;

#[derive(Resource)]
pub struct InstanceHandle(pub Handle<AudioInstance>);

#[derive(Resource)]
pub struct AudioAssetId(pub AssetId<AudioSource>);

/// Accumulated time delta (in seconds) for pending seek operations
///
/// When timeline_smooth_seeking is enabled, the delta is applied gradually;
/// when disabled, it's applied immediately.
#[derive(Resource)]
pub struct SeekDeltaTime(f32);

/// The duration of the audio
#[derive(Resource, Debug)]
pub struct AudioDuration(pub Duration);

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
                )
                    .run_if(project_loaded().and(resource_exists::<InstanceHandle>)),
            );
    }
}

#[derive(Error, Debug)]
pub enum LoadAudioError {
    #[error("unknown file format")]
    UnknownFormat,
    #[error("unsupported file format {0}")]
    UnsupportedFormat(&'static str),
    #[error("failed to read audio file")]
    Io(#[from] std::io::Error),
    #[error("failed to load audio source")]
    Load(#[from] FromFileError),
}

pub fn load_audio(path: PathBuf, commands: &mut Commands) -> Result<(), LoadAudioError> {
    let sound_data = std::fs::read(path)?;

    let is_supported = infer::audio::is_wav(sound_data.as_slice())
        || infer::audio::is_ogg(sound_data.as_slice())
        || infer::audio::is_flac(sound_data.as_slice())
        || infer::audio::is_wav(sound_data.as_slice());

    if !is_supported {
        return match infer::get(&sound_data) {
            None => Err(LoadAudioError::UnknownFormat),
            Some(file_type) => Err(LoadAudioError::UnsupportedFormat(file_type.mime_type())),
        };
    }

    let source = AudioSource {
        sound: StaticSoundData::from_cursor(Cursor::new(sound_data))?,
    };

    commands.insert_resource(Spectrogram(spectrogram::make_spectrogram(&source)));

    commands.queue(|world: &mut World| {
        world.insert_resource(AudioDuration(source.sound.duration()));
        world.resource_scope(|world, mut audios: Mut<Assets<AudioSource>>| {
            world.resource_scope(|world, audio: Mut<Audio>| {
                let handle = audios.add(source);
                world.insert_resource(AudioAssetId(handle.id()));
                let instance_handle = audio.play(handle).paused().handle();
                world.insert_resource(InstanceHandle(instance_handle));
            });
        });
    });

    Ok(())
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

// TODO: move this to separate plugin
/// When receiving [ResumeEvent], resume the audio instance
fn handle_resume_system(
    handle: Res<InstanceHandle>,
    mut paused: ResMut<Paused>,
    mut game_paused: ResMut<phichain_game::Paused>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut events: EventReader<ResumeEvent>,

    mut timing: ResMut<Timing>,
) {
    if let Some(instance) = audio_instances.get_mut(&handle.0) {
        for _ in events.read() {
            instance.resume(AudioTween::default());
            paused.0 = false;
            game_paused.0 = false;
            timing.resume();
        }
    }
}

/// Apply accumulated [`SeekDeltaTime`] to [`Timing`] and the audio instance
fn update_seek_system(
    handle: Res<InstanceHandle>,
    paused: Res<Paused>,
    time: Res<Time>,
    settings: Res<Persistent<EditorSettings>>,

    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut seek_delta_time: ResMut<SeekDeltaTime>,
    mut timing: ResMut<Timing>,
) {
    let delta = time.delta_secs();
    let now = timing.now();
    let seek_delta = seek_delta_time.0 * delta * 10.;
    timing.seek_to(now + seek_delta);
    seek_delta_time.0 -= seek_delta;
    // We directly seek the audio instance if it is not paused or if smooth seeking is disabled
    if (!paused.0 || !settings.general.timeline_smooth_seeking) && seek_delta_time.0.abs() > 0.0 {
        let final_time = timing.now() + seek_delta_time.0;
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
fn handle_seek_to_system(
    handle: Res<InstanceHandle>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut events: EventReader<SeekToEvent>,
    mut seek_delta_time: ResMut<SeekDeltaTime>,

    mut timing: ResMut<Timing>,
) {
    if let Some(instance) = audio_instances.get_mut(&handle.0) {
        for event in events.read() {
            instance.seek_to(event.0.max(0.0).into());
            timing.seek_to(event.0.max(0.0));
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
