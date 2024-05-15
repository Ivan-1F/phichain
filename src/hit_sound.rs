use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::{
    chart::note::{Note, NoteKind},
    timing::{BpmList, ChartTime},
};

pub struct HitSoundPlugin;

impl Plugin for HitSoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (add_marker_system, play_hit_sound_system).chain());
    }
}

#[derive(Component, Debug)]
struct PlayHitSound(NoteKind);

#[derive(Component, Debug)]
struct PlayedHitSound;

fn add_marker_system(
    mut commands: Commands,
    query: Query<(&Note, Entity), Without<PlayedHitSound>>,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
) {
    for (note, entity) in &query {
        if bpm_list.time_at(note.beat) < time.0 {
            commands.entity(entity).insert(PlayHitSound(note.kind));
        }
    }
}

fn play_hit_sound_system(
    mut commands: Commands,
    query: Query<(Entity, &PlayHitSound)>,
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
) {
    for (entity, hit_sound) in &query {
        commands.entity(entity).remove::<PlayHitSound>();
        commands.entity(entity).insert(PlayedHitSound);
        let path = match hit_sound.0 {
            NoteKind::Tap => "audio/HitSong0.ogg",
            NoteKind::Drag => "audio/HitSong1.ogg",
            NoteKind::Hold { hold_beat: _ } => "audio/HitSong0.ogg",
            NoteKind::Flick => "audio/HitSong2.ogg",
        };
        audio.play(asset_server.load(path));
    }
}
