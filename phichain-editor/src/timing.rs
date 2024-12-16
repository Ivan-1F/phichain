use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use phichain_chart::bpm_list::BpmList;

use crate::action::ActionRegistrationExt;
use crate::hotkey::{Hotkey, HotkeyContext, HotkeyExt};
use crate::identifier::{Identifier, IntoIdentifier};
use crate::project::project_loaded;
use crate::settings::EditorSettings;
use crate::tab::timeline::TimelineViewport;

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
                compute_bpm_list_system
                    .run_if(project_loaded().and_then(resource_changed::<BpmList>)),
            )
            .add_systems(
                Update,
                scroll_progress_control_system.run_if(project_loaded()),
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
) {
    if paused.0 {
        resume_events.send_default();
    } else {
        pause_events.send_default();
    }
}

/// Use ArrowLeft and ArrowRight to control the progress
fn progress_control_system(hotkey: HotkeyContext, mut events: EventWriter<SeekEvent>) {
    if hotkey.pressed(TimingHotkeys::Backward) {
        events.send(SeekEvent(-0.02));
    }
    if hotkey.pressed(TimingHotkeys::Forward) {
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

fn compute_bpm_list_system(mut bpm_list: ResMut<BpmList>) {
    bpm_list.compute();
}
