use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::project::project_loaded;
use crate::timing::Paused;
use crate::{
    chart::note::{Note, NoteKind},
    timing::{BpmList, ChartTime},
};

pub struct HitSoundPlugin;

impl Plugin for HitSoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, play_hit_sound_system.run_if(project_loaded()));
    }
}

#[derive(Component, Debug)]
struct PlayedHitSound;

fn play_hit_sound_system(
    mut commands: Commands,
    query: Query<(&Note, Entity, Option<&PlayedHitSound>)>,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    audio_settings: Res<crate::audio::AudioSettings>,
    paused: Res<Paused>,
) {
    for (note, entity, played) in &query {
        let note_time = bpm_list.time_at(note.beat);
        if note_time <= time.0 && time.0 - note_time < 0.05 && played.is_none() && !paused.0 {
            let path = match note.kind {
                NoteKind::Tap => "audio/click.ogg",
                NoteKind::Drag => "audio/drag.ogg",
                NoteKind::Hold { hold_beat: _ } => "audio/click.ogg",
                NoteKind::Flick => "audio/flick.ogg",
            };
            audio
                .play(asset_server.load(path))
                .with_volume(Volume::Amplitude(audio_settings.hit_sound_volume as f64));
            commands.entity(entity).insert(PlayedHitSound);
        } else if note_time > time.0 && played.is_some() {
            commands.entity(entity).remove::<PlayedHitSound>();
        }
    }
}
