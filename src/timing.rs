use bevy::prelude::*;

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

pub struct TimingPlugin;

impl Plugin for TimingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChartTime(0.0))
            .insert_resource(Paused(true))
            .add_event::<PauseEvent>()
            .add_event::<ResumeEvent>()
            .add_systems(Update, space_pause_resume_control);
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
