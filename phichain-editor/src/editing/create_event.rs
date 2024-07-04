use bevy::prelude::*;
use phichain_chart::bpm_list::BpmList;

use crate::editing::command::event::CreateEvent;
use crate::editing::command::EditorCommand;
use crate::editing::pending::Pending;
use crate::editing::DoCommandEvent;
use crate::project::project_loaded;
use crate::timeline::settings::TimelineSettings;
use crate::timeline::TimelineContext;
use crate::{selection::SelectedLine, tab::timeline::TimelineViewport};
use phichain_chart::event::{LineEvent, LineEventBundle, LineEventKind};

pub struct CreateEventPlugin;

impl Plugin for CreateEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (create_event_system, remove_pending_event_on_esc_system).run_if(project_loaded()),
        );
    }
}

fn create_event_system(
    mut commands: Commands,
    timeline: TimelineContext,
    keyboard: Res<ButtonInput<KeyCode>>,

    selected_line: Res<SelectedLine>,

    window_query: Query<&Window>,
    bpm_list: Res<BpmList>,

    timeline_viewport: Res<TimelineViewport>,
    timeline_settings: Res<TimelineSettings>,

    mut event: EventWriter<DoCommandEvent>,

    mut pending_event_query: Query<(&mut LineEvent, Entity), With<Pending>>,

    event_query: Query<(&LineEvent, &Parent), Without<Pending>>,
) {
    let window = window_query.single();
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let event_timeline_viewport = timeline_viewport.event_timeline_viewport();

    if !event_timeline_viewport.contains(cursor_position) {
        return;
    }

    let calc_event_attrs = || {
        let time = timeline.y_to_time(cursor_position.y);
        let beat = bpm_list.beat_at(time).value();
        let beat = timeline_settings.attach(beat);

        let track = ((cursor_position.x - event_timeline_viewport.min.x)
            / (event_timeline_viewport.width() / 5.0))
            .ceil() as u8;

        (track, beat)
    };

    if let Ok((mut pending_event, _)) = pending_event_query.get_single_mut() {
        let (track, beat) = calc_event_attrs();
        pending_event.end_beat =
            beat.max(pending_event.start_beat + timeline_settings.minimum_beat());
        pending_event.kind = LineEventKind::try_from(track).expect("Unknown event track");
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        if let Ok((pending_event, entity)) = pending_event_query.get_single() {
            // inherit event's start & end value from neighbor events
            let mut new_event = *pending_event;
            let mut events = event_query.iter().collect::<Vec<_>>();
            events.sort_by_key(|x| x.0.start_beat);
            if let Some(last_event) = events
                .iter()
                .filter(|(e, _)| e.kind == pending_event.kind)
                .filter(|(_, p)| p.get() == selected_line.0)
                .take_while(|(e, _)| e.start_beat <= pending_event.start_beat)
                .map(|x| x.0)
                .next()
            {
                new_event.start = last_event.end;
            }
            events.reverse();
            if let Some(next_event) = events
                .iter()
                .filter(|(e, _)| e.kind == pending_event.kind)
                .filter(|(_, p)| p.get() == selected_line.0)
                .take_while(|(e, _)| e.end_beat >= pending_event.end_beat)
                .map(|x| x.0)
                .next()
            {
                new_event.end = next_event.start;
            }
            commands.entity(entity).despawn();
            event.send(DoCommandEvent(EditorCommand::CreateEvent(
                CreateEvent::new(selected_line.0, new_event),
            )));
        } else {
            let (track, beat) = calc_event_attrs();
            let kind = LineEventKind::try_from(track).expect("Unknown event track");
            commands.entity(selected_line.0).with_children(|parent| {
                parent.spawn((
                    LineEventBundle::new(LineEvent {
                        kind,
                        start: 0.0,
                        end: 0.0,
                        start_beat: beat,
                        end_beat: beat + timeline_settings.minimum_beat(),
                        easing: Default::default(),
                    }),
                    Pending,
                ));
            });
        }
    }
}

fn remove_pending_event_on_esc_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<Entity, (With<Pending>, With<LineEvent>)>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        for entity in &query {
            commands.entity(entity).despawn();
        }
    }
}
