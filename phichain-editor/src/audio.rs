use std::f32;
use std::time::Duration;
use std::{io::Cursor, path::PathBuf};

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_persistent::prelude::*;

use crate::settings::EditorSettings;
use crate::timing::{SeekToEvent, Timing};
use crate::utils::compat::ControlKeyExt;
use crate::{
    project::project_loaded,
    timing::{PauseEvent, Paused, ResumeEvent, SeekEvent},
};

#[derive(Resource)]
pub struct InstanceHandle(pub Handle<AudioInstance>);

#[derive(Resource)]
pub struct AudioAssetId(pub AssetId<AudioSource>);

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

pub fn load_audio(path: PathBuf, commands: &mut Commands) {
    let sound_data = std::fs::read(path).unwrap();
    let source = AudioSource {
        sound: StaticSoundData::from_cursor(Cursor::new(sound_data)).unwrap(),
    };
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

fn update_seek_system(
    handle: Res<InstanceHandle>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    time: Res<Time>,
    mut seek_delta_time: ResMut<SeekDeltaTime>,

    mut timing: ResMut<Timing>,
) {
    let delta = time.delta_secs();
    let now = timing.now();
    timing.seek_to(now + seek_delta_time.0 * delta * 10.);
    if let Some(instance) = audio_instances.get_mut(&handle.0) {
        instance.seek_to(timing.now() as f64);
    }
    seek_delta_time.0 -= seek_delta_time.0 * delta * 10.;
}

// TODO: move this to separate plugin
/// When receiving [SeekEvent], seek the audio instance
fn handle_seek_system(
    handle: Res<InstanceHandle>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut events: EventReader<SeekEvent>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut seek_target_time: ResMut<SeekDeltaTime>,
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
            if let Some(position) = instance.state().position() {
                seek_target_time.0 = event.0 * factor;
                instance.seek_to(position + seek_target_time.0 as f64);
            }
        }
    }
}

// TODO: move this to separate plugin
/// When receiving [SeekToEvent], seek the audio instance
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
