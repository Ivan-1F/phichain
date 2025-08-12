use crate::action::ActionRegistrationExt;
use crate::hotkey::Hotkey;
use crate::settings::EditorSettings;
use crate::timing::{ChartTime, Paused};
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_persistent::Persistent;
use phichain_assets::AudioAssets;
use phichain_chart::beat;
use phichain_chart::bpm_list::BpmList;
use phichain_game::GameSet;

#[derive(Resource, Debug, Default)]
struct MetronomeState {
    last_beat_played: Option<i32>,
}

#[derive(Event, Default)]
pub struct ToggleMetronomeEvent;

pub struct MetronomePlugin;

impl Plugin for MetronomePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ToggleMetronomeEvent>()
            .init_resource::<MetronomeState>()
            .add_systems(Update, play_metronome_system.in_set(GameSet))
            .add_action(
                "phichain.metronome.toggle",
                toggle_metronome_action,
                Some(Hotkey::new(KeyCode::KeyM, vec![])),
            );
    }
}

fn play_metronome_system(
    time: Res<ChartTime>,
    bmp_list: Res<BpmList>,
    assets: Res<AudioAssets>,
    audio: Res<Audio>,
    settings: Res<Persistent<EditorSettings>>,
    paused: Res<Paused>,
    mut metronome_state: ResMut<MetronomeState>,
) {
    if !settings.audio.metronome_enabled || paused.0 {
        metronome_state.last_beat_played = None;
        return;
    }

    let current_time = time.0;
    let current_beat = bmp_list.beat_at(current_time);

    let should_play = match metronome_state.last_beat_played {
        None => true,
        Some(last_beat) => current_beat.beat() != last_beat,
    };

    if should_play {
        let beat_time = bmp_list.time_at(beat!(current_beat.beat()));
        let time_diff = current_time - beat_time;

        // only play if we're close to the beat (within 80ms after the beat)
        if (0.0..0.08).contains(&time_diff) {
            audio
                .play(assets.metronome.clone())
                .with_volume(Volume::Amplitude(settings.audio.metronome_volume as f64));

            metronome_state.last_beat_played = Some(current_beat.beat());
        }
    }
}

fn toggle_metronome_action(
    mut settings: ResMut<Persistent<EditorSettings>>,
    mut metronome_state: ResMut<MetronomeState>,
) -> Result<()> {
    settings.audio.metronome_enabled = !settings.audio.metronome_enabled;
    settings.persist()?;

    metronome_state.last_beat_played = None;
    Ok(())
}
