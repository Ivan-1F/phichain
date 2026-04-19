use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_persistent::Persistent;
use phichain_assets::HitSoundAssets;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::note::{Note, NoteKind};

use crate::project::project_loaded;
use crate::settings::EditorSettings;
use crate::timing::ChartTime;
use crate::timing::Paused;

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
    assets: Res<HitSoundAssets>,
    audio: Res<Audio>,
    settings: Res<Persistent<EditorSettings>>,
    paused: Res<Paused>,
) {
    for (note, entity, played) in &query {
        let note_time = bpm_list.time_at(note.beat);
        if note_time <= time.0 && time.0 - note_time < 0.05 && played.is_none() && !paused.0 {
            let handle = match note.kind {
                NoteKind::Tap => assets.tap.clone(),
                NoteKind::Drag => assets.drag.clone(),
                NoteKind::Hold { .. } => assets.tap.clone(),
                NoteKind::Flick => assets.flick.clone(),
            };
            audio
                .play(handle)
                .with_volume(crate::utils::audio::amplitude_to_db(
                    settings.audio.hit_sound_volume,
                ));
            commands.entity(entity).insert(PlayedHitSound);
        } else if note_time > time.0 && played.is_some() {
            commands.entity(entity).remove::<PlayedHitSound>();
        }
    }
}
