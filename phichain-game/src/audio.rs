use std::io::Cursor;
use std::path::PathBuf;
use std::time::Duration;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use thiserror::Error;

#[derive(Resource)]
pub struct InstanceHandle(pub Handle<AudioInstance>);

#[derive(Resource)]
pub struct AudioAssetId(pub AssetId<AudioSource>);

#[derive(Resource, Debug)]
pub struct AudioDuration(pub Duration);

#[derive(Debug, Clone, Default)]
pub struct AudioBytes(pub Vec<u8>);

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

pub fn open_audio(path: PathBuf) -> Result<AudioBytes, LoadAudioError> {
    let bytes = std::fs::read(path)?;

    let is_supported = infer::audio::is_wav(bytes.as_slice())
        || infer::audio::is_ogg(bytes.as_slice())
        || infer::audio::is_flac(bytes.as_slice())
        || infer::audio::is_mp3(bytes.as_slice());

    if !is_supported {
        return match infer::get(&bytes) {
            None => Err(LoadAudioError::UnknownFormat),
            Some(file_type) => Err(LoadAudioError::UnsupportedFormat(file_type.mime_type())),
        };
    }

    Ok(AudioBytes(bytes))
}

pub fn load_audio(bytes: AudioBytes, commands: &mut Commands) -> Result<(), LoadAudioError> {
    let AudioBytes(bytes) = bytes;

    let source = AudioSource {
        sound: StaticSoundData::from_cursor(Cursor::new(bytes))?,
    };
    let duration = source.sound.duration();

    commands.queue(move |world: &mut World| {
        world.insert_resource(AudioDuration(duration));
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
