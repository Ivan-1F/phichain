use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::timing::{ChartTime, PauseEvent, Paused, ResumeEvent, SeekEvent};

#[derive(Resource)]
struct InstanceHandle(Handle<AudioInstance>);

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_kira_audio::AudioPlugin)
            .add_systems(Startup, setup_audio_system)
            .add_systems(Update, handle_pause_system)
            .add_systems(Update, handle_resume_system)
            .add_systems(Update, handle_seek_system)
            .add_systems(Update, update_time_system);
    }
}

/// Setup music
fn setup_audio_system(mut commands: Commands, asset_server: Res<AssetServer>, audio: Res<Audio>) {
    let handle = audio.play(asset_server.load("audio.mp3")).paused().handle();
    commands.insert_resource(InstanceHandle(handle));
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
) {
    if let Some(instance) = audio_instances.get_mut(&handle.0) {
        for event in events.read() {
            match instance.state() {
                PlaybackState::Paused { position }
                | PlaybackState::Pausing { position }
                | PlaybackState::Playing { position }
                | PlaybackState::Stopping { position } => {
                    instance.seek_to((position as f32 + event.0).max(0.0).into());
                }
                PlaybackState::Queued | PlaybackState::Stopped => {}
            }
        }
    }
}

/// Sync the [ChartTime] with the audio instance
fn update_time_system(
    handle: Res<InstanceHandle>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut time: ResMut<ChartTime>,
) {
    if let Some(instance) = audio_instances.get_mut(&handle.0) {
        time.0 = instance.state().position().unwrap_or_default() as f32
    }
}
