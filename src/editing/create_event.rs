use bevy::prelude::*;

use crate::chart::event::{LineEvent, LineEventBundle, LineEventKind};
use crate::editing::command::event::CreateEvent;
use crate::editing::command::EditorCommand;
use crate::editing::pending::Pending;
use crate::editing::DoCommandEvent;
use crate::{
    selection::SelectedLine,
    tab::timeline::{Timeline, TimelineSettings, TimelineViewport},
    timing::BpmList,
};

pub fn create_event_system(
    mut commands: Commands,
    timeline: Timeline,
    keyboard: Res<ButtonInput<KeyCode>>,

    selected_line: Res<SelectedLine>,

    window_query: Query<&Window>,
    bpm_list: Res<BpmList>,

    timeline_viewport: Res<TimelineViewport>,
    timeline_settings: Res<TimelineSettings>,

    mut event: EventWriter<DoCommandEvent>,

    mut pending_event_query: Query<(&mut LineEvent, Entity), With<Pending>>,
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
        let mut beat = bpm_list.beat_at(time);
        beat.attach_to_beat_line(timeline_settings.density);

        let track = ((cursor_position.x - event_timeline_viewport.min.x)
            / (event_timeline_viewport.width() / 5.0))
            .ceil() as u32;

        (track, beat)
    };

    if let Ok((mut pending_event, _)) = pending_event_query.get_single_mut() {
        let (_, beat) = calc_event_attrs();
        pending_event.end_beat =
            beat.max(pending_event.start_beat + timeline_settings.minimum_beat());
        // pending_note.x = x * CANVAS_WIDTH;
    }

    if keyboard.just_pressed(KeyCode::KeyR) {
        if let Ok((pending_event, entity)) = pending_event_query.get_single() {
            commands.entity(entity).despawn();
            event.send(DoCommandEvent(EditorCommand::CreateEvent(
                CreateEvent::new(selected_line.0, *pending_event),
            )));
        } else {
            let (track, beat) = calc_event_attrs();
            let kind = match track {
                1 => LineEventKind::X,
                2 => LineEventKind::Y,
                3 => LineEventKind::Rotation,
                4 => LineEventKind::Opacity,
                5 => LineEventKind::Speed,
                _ => unreachable!(),
            };
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
