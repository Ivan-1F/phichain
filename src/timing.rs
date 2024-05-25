use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use serde::{Deserialize, Deserializer, Serialize};

use crate::action::ActionRegistrationExt;
use crate::hotkey::HotkeyRegistrationExt;
use crate::settings::EditorSettings;
use crate::tab::timeline::TimelineViewport;
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

/// Seek the chart by certain delta
#[derive(Event, Default)]
pub struct SeekEvent(pub f32);

/// Seek the chart to certain point
#[derive(Event, Default)]
pub struct SeekToEvent(pub f32);

pub struct TimingPlugin;

impl Plugin for TimingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChartTime(0.0))
            .insert_resource(Paused(true))
            .add_event::<PauseEvent>()
            .add_event::<ResumeEvent>()
            .add_event::<SeekEvent>()
            .add_event::<SeekToEvent>()
            .add_systems(Update, progress_control_system.run_if(project_loaded()))
            .add_systems(
                Update,
                scroll_progress_control_system.run_if(project_loaded()),
            )
            .register_action("phichain.toggle", toggle_system)
            .register_hotkey("phichain.toggle", vec![KeyCode::Space]);
    }
}

fn toggle_system(
    paused: Res<Paused>,
    mut pause_events: EventWriter<PauseEvent>,
    mut resume_events: EventWriter<ResumeEvent>,
) {
    if paused.0 {
        resume_events.send_default();
    } else {
        pause_events.send_default();
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
    mut seek_events: EventWriter<SeekEvent>,
    viewport: Res<TimelineViewport>,
    window_query: Query<&Window>,

    settings: Res<Persistent<EditorSettings>>,
) {
    let window = window_query.single();
    if window
        .cursor_position()
        .is_some_and(|p| viewport.0.contains(p))
    {
        for ev in wheel_events.read() {
            seek_events.send(SeekEvent(
                ev.y / 5000.0 * settings.general.timeline_scroll_sensitivity,
            ));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmPoint {
    pub beat: Beat,
    pub bpm: f32,

    #[serde(skip_serializing, default)]
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

#[derive(Resource, Debug, Clone, Serialize)]
pub struct BpmList(pub Vec<BpmPoint>);

impl<'de> Deserialize<'de> for BpmList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let points = Vec::<BpmPoint>::deserialize(deserializer)?;
        let mut bpm_list = BpmList::new(points);
        bpm_list.compute();

        Ok(bpm_list)
    }
}

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

    pub fn compute(&mut self) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_at() {
        let bpm_list = BpmList::new(vec![
            BpmPoint::new(Beat::ZERO, 120.0),
            BpmPoint::new(Beat::from(4.0), 240.0),
        ]);
        assert_eq!(bpm_list.time_at(Beat::ZERO), 0.0);
        assert_eq!(bpm_list.time_at(Beat::ONE), 0.5);
        assert_eq!(bpm_list.time_at(Beat::from(2.0)), 1.0);
        assert_eq!(bpm_list.time_at(Beat::from(3.0)), 1.5);
        assert_eq!(bpm_list.time_at(Beat::from(4.0)), 2.0);
        assert_eq!(bpm_list.time_at(Beat::from(5.0)), 2.0 + 0.25);
        assert_eq!(bpm_list.time_at(Beat::from(6.0)), 2.0 + 0.25 * 2.0);
        assert_eq!(bpm_list.time_at(Beat::from(7.0)), 2.0 + 0.25 * 3.0);
        assert_eq!(bpm_list.time_at(Beat::from(8.0)), 2.0 + 0.25 * 4.0);
    }

    #[test]
    fn test_beat_at() {
        let bpm_list = BpmList::new(vec![
            BpmPoint::new(Beat::ZERO, 120.0),
            BpmPoint::new(Beat::from(4.0), 240.0),
        ]);
        assert_eq!(bpm_list.beat_at(0.0), Beat::ZERO);
        assert_eq!(bpm_list.beat_at(0.5), Beat::ONE);
        assert_eq!(bpm_list.beat_at(1.0), Beat::from(2.0));
        assert_eq!(bpm_list.beat_at(1.5), Beat::from(3.0));
        assert_eq!(bpm_list.beat_at(2.0), Beat::from(4.0));
        assert_eq!(bpm_list.beat_at(2.0 + 0.25), Beat::from(5.0));
        assert_eq!(bpm_list.beat_at(2.0 + 0.25 * 2.0), Beat::from(6.0));
        assert_eq!(bpm_list.beat_at(2.0 + 0.25 * 3.0), Beat::from(7.0));
        assert_eq!(bpm_list.beat_at(2.0 + 0.25 * 4.0), Beat::from(8.0));
    }

    #[test]
    fn test_bpm_list_serialization() {
        let bpm_list = BpmList::new(vec![
            BpmPoint::new(Beat::ZERO, 120.0),
            BpmPoint::new(Beat::ONE, 240.0),
        ]);

        let string = serde_json::to_string(&bpm_list).unwrap();
        assert_eq!(
            string,
            "[{\"beat\":[0,0,1],\"bpm\":120.0},{\"beat\":[1,0,1],\"bpm\":240.0}]".to_string()
        );

        let deserialized: BpmList = serde_json::from_str(
            "[{\"beat\":[0,0,1],\"bpm\":120.0},{\"beat\":[1,0,1],\"bpm\":240.0}]",
        )
        .unwrap();

        let first = &deserialized.0[0];
        let second = &deserialized.0[1];

        assert_eq!(first.beat, Beat::ZERO);
        assert_eq!(first.bpm, 120.0);
        assert_eq!(first.time, 0.0);

        assert_eq!(second.beat, Beat::ONE);
        assert_eq!(second.bpm, 240.0);
        assert_eq!(second.time, 0.5);
    }
}
