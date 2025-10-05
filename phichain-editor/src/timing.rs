use crate::action::ActionRegistrationExt;
use crate::hotkey::{Hotkey, HotkeyContext, HotkeyExt};
use crate::identifier::{Identifier, IntoIdentifier};
use crate::project::project_loaded;
use crate::settings::EditorSettings;
use bevy::prelude::*;
use bevy_kira_audio::AudioInstance;
use bevy_persistent::Persistent;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::offset::Offset;
use phichain_game::audio::InstanceHandle;

/// Represents the current time in seconds
#[derive(Resource)]
pub struct ChartTime(pub f32);

/// Represents if the editor is paused
#[derive(Resource)]
pub struct Paused(pub bool);

/// Pause the chart
#[derive(Event, Default)]
pub struct PauseEvent;

/// Resume the chart
#[derive(Event, Default)]
pub struct ResumeEvent;

/// Seek the chart by certain delta
#[derive(Event, Default)]
pub struct SeekEvent(pub f32);

/// Seek the chart to certain point
#[derive(Event, Default)]
pub struct SeekToEvent(pub f32);

pub enum TimingHotkeys {
    Forward,
    Backward,
}

impl IntoIdentifier for TimingHotkeys {
    fn into_identifier(self) -> Identifier {
        match self {
            TimingHotkeys::Forward => "phichain.forward".into(),
            TimingHotkeys::Backward => "phichain.backward".into(),
        }
    }
}

pub struct TimingPlugin;

impl Plugin for TimingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChartTime(0.0))
            .insert_resource(Paused(true))
            .add_event::<PauseEvent>()
            .add_event::<ResumeEvent>()
            .add_event::<SeekEvent>()
            .add_event::<SeekToEvent>()
            .add_hotkey(
                TimingHotkeys::Backward,
                Hotkey::new(KeyCode::BracketLeft, vec![]),
            )
            .add_hotkey(
                TimingHotkeys::Forward,
                Hotkey::new(KeyCode::BracketRight, vec![]),
            )
            .add_systems(Update, progress_control_system.run_if(project_loaded()))
            .add_systems(
                Update,
                compute_bpm_list_system.run_if(project_loaded().and(resource_changed::<BpmList>)),
            )
            .insert_resource(Timing::new())
            .add_systems(
                PreUpdate,
                update_time_system.run_if(project_loaded().and(resource_exists::<InstanceHandle>)),
            )
            .add_action(
                "phichain.pause_resume",
                toggle_system,
                Some(Hotkey::new(KeyCode::Space, vec![])),
            );
    }
}

fn toggle_system(
    paused: Res<Paused>,
    mut pause_events: EventWriter<PauseEvent>,
    mut resume_events: EventWriter<ResumeEvent>,
) -> Result {
    if paused.0 {
        resume_events.write_default();
    } else {
        pause_events.write_default();
    }

    Ok(())
}

/// Use ArrowLeft and ArrowRight to control the progress
fn progress_control_system(hotkey: HotkeyContext, mut events: EventWriter<SeekEvent>) {
    if hotkey.pressed(TimingHotkeys::Backward) {
        events.write(SeekEvent(-0.02));
    }
    if hotkey.pressed(TimingHotkeys::Forward) {
        events.write(SeekEvent(0.02));
    }
}

fn compute_bpm_list_system(mut bpm_list: ResMut<BpmList>) {
    bpm_list.compute();
}

/// Controls global editor timing
///
/// Credits:
///
/// - https://mivik.moe/2023/research/sasa
/// - https://github.com/TeamFlos/phira/blob/main/prpr/src/time.rs
#[derive(Debug, Clone, Resource)]
pub struct Timing {
    start_time: f32,
    pause_time: Option<f32>,

    real_time: f32,

    speed: f32,
    wait: f32,
}

impl Timing {
    fn new() -> Self {
        Self {
            start_time: 0.0,
            pause_time: Some(0.0),

            real_time: 0.0,

            speed: 1.0,
            wait: f32::NEG_INFINITY,
        }
    }

    pub fn wait(&mut self) {
        self.wait = self.real_time + 0.1;
    }

    #[allow(dead_code)]
    pub fn dont_wait(&mut self) {
        self.wait = f32::NEG_INFINITY;
    }

    #[must_use]
    pub fn now(&self) -> f32 {
        (self.pause_time.unwrap_or(self.real_time) - self.start_time) * self.speed
    }

    pub fn update(&mut self, real_time: f32, music_time: f32) {
        self.real_time = real_time;
        if self.real_time > self.wait && self.pause_time.is_none() {
            self.start_time -= (music_time - self.now()) * 3e-3;
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn paused(&self) -> bool {
        self.pause_time.is_some()
    }

    pub fn pause(&mut self) {
        self.pause_time = Some(self.real_time);
    }

    pub fn resume(&mut self) {
        self.start_time += self.real_time - self.pause_time.take().unwrap();
        self.wait();
    }

    pub fn seek_to(&mut self, pos: f32) {
        self.start_time = self.pause_time.unwrap_or(self.real_time) - pos / self.speed;
        self.wait();
    }

    pub fn set_speed(&mut self, new_speed: f32) {
        let old_speed = self.speed;

        let t = self.pause_time.unwrap_or(self.real_time);

        self.start_time = t - (t - self.start_time) * old_speed / new_speed;

        self.speed = new_speed;

        self.wait();
    }
}

/// Update [`phichain_game::ChartTime`] (for `phichain-game`) and [`ChartTime`] (for `phichain-editor`) with [`Timing`]
///
/// This will run on [`PreUpdate`], before [`phichain_game::GameSet`]
pub fn update_time_system(
    mut timing: ResMut<Timing>,
    time: Res<Time>,
    settings: Res<Persistent<EditorSettings>>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    handle: Res<InstanceHandle>,
    offset: Res<Offset>,

    mut chart_time: ResMut<ChartTime>,
    mut game_time: ResMut<phichain_game::ChartTime>,
) {
    timing.set_speed(settings.audio.playback_rate);

    if let Some(instance) = audio_instances.get_mut(&handle.0) {
        let value = instance.state().position().unwrap_or_default() as f32;
        timing.update(time.elapsed_secs(), value);

        let now = timing.now() - offset.0 / 1000.0;

        chart_time.0 = now;
        game_time.0 = now;
    }
}
