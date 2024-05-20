use std::time::Duration;
use std::{io::Cursor, path::PathBuf};

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use serde::{Deserialize, Serialize};

use crate::timing::SeekToEvent;
use crate::{
    project::project_loaded,
    timing::{ChartTime, PauseEvent, Paused, ResumeEvent, SeekEvent},
};

/// Chart offset in milliseconds
#[derive(Debug, Default, Resource, Serialize, Deserialize)]
pub struct Offset(pub f32);

#[derive(Resource)]
struct InstanceHandle(Handle<AudioInstance>);

/// The duration of the audio
#[derive(Resource, Debug)]
pub struct AudioDuration(pub Duration);

#[derive(Resource)]
pub struct AudioSettings {
    pub music_volume: f32,
    pub hit_sound_volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            music_volume: 1.0,
            hit_sound_volume: 1.0,
        }
    }
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AudioSettings::default())
            .add_plugins(bevy_kira_audio::AudioPlugin)
            .add_systems(
                Update,
                (
                    handle_pause_system,
                    handle_resume_system,
                    handle_seek_system,
                    handle_seek_to_system,
                    update_time_system,
                    update_volume_system,
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
            if keyboard.pressed(KeyCode::ControlLeft) {
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
    offset: Res<Offset>,
) {
    if let Some(instance) = audio_instances.get_mut(&handle.0) {
        time.0 = instance.state().position().unwrap_or_default() as f32 + offset.0 / 1000.0;
    }
}

fn update_volume_system(
    handle: Res<InstanceHandle>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    audio_settings: Res<AudioSettings>,
) {
    if let Some(instance) = audio_instances.get_mut(&handle.0) {
        instance.set_volume(
            Volume::Amplitude(audio_settings.music_volume as f64),
            AudioTween::default(),
        );
    }
}
