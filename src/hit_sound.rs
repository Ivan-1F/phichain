use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::assets::AudioAssets;
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

#[allow(clippy::too_many_arguments)]
fn play_hit_sound_system(
    mut commands: Commands,
    query: Query<(&Note, Entity, Option<&PlayedHitSound>)>,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
    assets: Res<AudioAssets>,
    audio: Res<Audio>,
    audio_settings: Res<crate::audio::AudioSettings>,
    paused: Res<Paused>,
) {
    for (note, entity, played) in &query {
        let note_time = bpm_list.time_at(note.beat);
        if note_time <= time.0 && time.0 - note_time < 0.05 && played.is_none() && !paused.0 {
            let handle = match note.kind {
                NoteKind::Tap => assets.click.clone(),
                NoteKind::Drag => assets.drag.clone(),
                NoteKind::Hold { .. } => assets.click.clone(),
                NoteKind::Flick => assets.flick.clone(),
            };
            audio
                .play(handle)
                .with_volume(Volume::Amplitude(audio_settings.hit_sound_volume as f64));
            commands.entity(entity).insert(PlayedHitSound);
        } else if note_time > time.0 && played.is_some() {
            commands.entity(entity).remove::<PlayedHitSound>();
        }
    }
}
