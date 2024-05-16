use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{chart::beat::Beat, project::project_loaded};

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

/// Seek the chart
#[derive(Event, Default)]
pub struct SeekEvent(pub f32);

pub struct TimingPlugin;

impl Plugin for TimingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChartTime(0.0))
            .insert_resource(Paused(true))
            .add_event::<PauseEvent>()
            .add_event::<ResumeEvent>()
            .add_event::<SeekEvent>()
            .add_systems(Update, space_pause_resume_control.run_if(project_loaded()))
            .add_systems(Update, progress_control_system.run_if(project_loaded()))
            .add_systems(
                Update,
                scroll_progress_control_system.run_if(project_loaded()),
            );
    }
}

/// Use ArrowLeft and ArrowRight to control the progress
fn progress_control_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut events: EventWriter<SeekEvent>,
) {
    if keyboard.pressed(KeyCode::ArrowLeft) {
        events.send(SeekEvent(-0.02));
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        events.send(SeekEvent(0.02));
    }
}

/// Scroll on the timeline to control the progress
fn scroll_progress_control_system(
    mut wheel_events: EventReader<MouseWheel>,
    mut events: EventWriter<SeekEvent>,
) {
    for ev in wheel_events.read() {
        events.send(SeekEvent(ev.y / 500.0));
    }
}

/// Toggle pause state when pressing space
fn space_pause_resume_control(
    keyboard: Res<ButtonInput<KeyCode>>,
    paused: Res<Paused>,
    mut pause_events: EventWriter<PauseEvent>,
    mut resume_events: EventWriter<ResumeEvent>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        if paused.0 {
            resume_events.send_default();
        } else {
            pause_events.send_default();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmPoint {
    beat: Beat,
    bpm: f32,

    time: f32,
}

impl BpmPoint {
    pub fn new(beat: Beat, bpm: f32) -> Self {
        Self {
            beat,
            bpm,
            time: 0.0,
        }
    }
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct BpmList(Vec<BpmPoint>);

impl Default for BpmList {
    fn default() -> Self {
        Self::new(vec![BpmPoint::new(Beat::ZERO, 120.0)])
    }
}

impl BpmList {
    pub fn new(points: Vec<BpmPoint>) -> Self {
        let mut list = Self(points);
        list.compute();
        list
    }

    fn compute(&mut self) {
        let mut time = 0.0;
        let mut last_beat = 0.0;
        let mut last_bpm = -1.0;
        for point in &mut self.0 {
            if last_bpm != -1.0 {
                time += (point.beat.value() - last_beat) * (60.0 / last_bpm);
            }
            last_beat = point.beat.value();
            last_bpm = point.bpm;
            point.time = time;
        }
    }

    pub fn time_at(&self, beat: Beat) -> f32 {
        let point = self
            .0
            .iter()
            .take_while(|p| p.beat.value() < beat.value())
            .last()
            .or_else(|| self.0.first())
            .expect("No bpm points available");

        point.time + (beat.value() - point.beat.value()) * (60.0 / point.bpm)
    }

    pub fn beat_at(&self, time: f32) -> Beat {
        let point = self
            .0
            .iter()
            .take_while(|p| p.time <= time)
            .last()
            .or_else(|| self.0.first())
            .expect("No bpm points available");

        Beat::from(point.beat.value() + (time - point.time) * point.bpm / 60.0)
    }
}
