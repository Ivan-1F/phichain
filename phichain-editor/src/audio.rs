use std::time::Duration;
use std::{io::Cursor, path::PathBuf};

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_persistent::prelude::*;
use phichain_chart::offset::Offset;

use crate::settings::EditorSettings;
use crate::timing::SeekToEvent;
use crate::utils::compat::ControlKeyExt;
use crate::{
    project::project_loaded,
    timing::{ChartTime, PauseEvent, Paused, ResumeEvent, SeekEvent},
};

#[derive(Resource)]
pub struct InstanceHandle(Handle<AudioInstance>);

#[derive(Resource)]
pub struct AudioAssetId(pub AssetId<AudioSource>);

/// The duration of the audio
#[derive(Resource, Debug)]
pub struct AudioDuration(pub Duration);

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_kira_audio::AudioPlugin).add_systems(
            Update,
            (
                handle_pause_system,
                handle_resume_system,
                handle_seek_system,
                handle_seek_to_system,
                update_time_system,
                update_volume_system,
                update_playback_rate_system,
            )
                .run_if(project_loaded().and_then(resource_exists::<InstanceHandle>)),
        );
    }
}

pub fn load_audio(path: PathBuf, commands: &mut Commands) {
    let sound_data = std::fs::read(path).unwrap();
    let source = AudioSource {
        sound: StaticSoundData::from_cursor(
            Cursor::new(sound_data),
            StaticSoundSettings::default(),
        )
        .unwrap(),
    };
    commands.add(|world: &mut World| {
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
}

/// When receiving [PauseEvent], pause the audio instance
fn handle_pause_system(
    handle: Res<InstanceHandle>,
    mut paused: ResMut<Paused>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut events: EventReader<PauseEvent>,
) {
    if let Some(instance) = audio_instances.get_mut(&handle.0) {
        for _ in events.read() {
            instance.pause(AudioTween::default());
            paused.0 = true;
        }
    }
}

/// When receiving [ResumeEvent], resume the audio instance
fn handle_resume_system(
    handle: Res<InstanceHandle>,
    mut paused: ResMut<Paused>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut events: EventReader<ResumeEvent>,
) {
    if let Some(instance) = audio_instances.get_mut(&handle.0) {
        for _ in events.read() {
            instance.resume(AudioTween::default());
            paused.0 = false;
        }
    }
}

/// When receiving [SeekEvent], seek the audio instance
fn handle_seek_system(
    handle: Res<InstanceHandle>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut events: EventReader<SeekEvent>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if let Some(instance) = audio_instances.get_mut(&handle.0) {
        for event in events.read() {
            // holding Control will seek faster and holding Alt will seek slower
            let mut factor = 1.0;
            if keyboard.pressed(KeyCode::control()) {
                factor *= 2.0;
            }
            if keyboard.pressed(KeyCode::AltLeft) {
                factor /= 2.0;
            }
            match instance.state() {
                PlaybackState::Paused { position }
                | PlaybackState::Pausing { position }
                | PlaybackState::Playing { position }
                | PlaybackState::Stopping { position } => {
                    instance.seek_to((position as f32 + event.0 * factor).max(0.0).into());
                }
                PlaybackState::Queued | PlaybackState::Stopped => {}
            }
        }
    }
}

/// When receiving [SeekToEvent], seek the audio instance
fn handle_seek_to_system(
    handle: Res<InstanceHandle>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut events: EventReader<SeekToEvent>,
) {
    if let Some(instance) = audio_instances.get_mut(&handle.0) {
        for event in events.read() {
            instance.seek_to(event.0.max(0.0).into());
        }
    }
}

/// Sync the [ChartTime] with the audio instance
fn update_time_system(
    handle: Res<InstanceHandle>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut time: ResMut<ChartTime>,
    mut game_time: ResMut<phichain_game::ChartTime>,
    offset: Res<Offset>,
) {
    if let Some(instance) = audio_instances.get_mut(&handle.0) {
        let value = instance.state().position().unwrap_or_default() as f32 + offset.0 / 1000.0;
        time.0 = value;
        game_time.0 = value;
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
