use bevy::prelude::*;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::easing::Easing;
use phichain_game::Pending;

use crate::editing::history::Edits;
use crate::hotkey::{Hotkey, HotkeyContext, HotkeyExt};
use crate::identifier::{Identifier, IntoIdentifier};
use crate::schedule::EditorSet;
use crate::timeline::container::TimelineLayout;
use crate::timeline::event::EventTimeline;
use crate::timeline::pending::{spawn_pending, TimelinePending};
use crate::timeline::pending_status::{PendingHint, PendingStatus};
use crate::timeline::TimelineContext;
use crate::utils::convert::BevyEguiConvert;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
use phichain_game::event::{EventIndex, EventOf};

enum CreateEventHotkeys {
    PlaceTransitionEvent,
    PlaceConstantEvent,
}

impl IntoIdentifier for CreateEventHotkeys {
    fn into_identifier(self) -> Identifier {
        match self {
            CreateEventHotkeys::PlaceTransitionEvent => "phichain.place_transition_event".into(),
            CreateEventHotkeys::PlaceConstantEvent => "phichain.place_constant_event".into(),
        }
    }
}

fn requested_event_value(hotkey: &HotkeyContext) -> Option<LineEventValue> {
    if hotkey.just_pressed(CreateEventHotkeys::PlaceTransitionEvent) {
        Some(LineEventValue::transition(0.0, 0.0, Easing::Linear))
    } else if hotkey.just_pressed(CreateEventHotkeys::PlaceConstantEvent) {
        Some(LineEventValue::constant(0.0))
    } else {
        None
    }
}

pub struct CreateEventPlugin;

impl Plugin for CreateEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, create_event_system.in_set(EditorSet::Edit))
            .add_hotkey(
                CreateEventHotkeys::PlaceTransitionEvent,
                Hotkey::new(KeyCode::KeyR, vec![]),
            )
            .add_hotkey(
                CreateEventHotkeys::PlaceConstantEvent,
                Hotkey::new(KeyCode::KeyQ, vec![]),
            );
    }
}

fn create_event_system(
    mut commands: Commands,
    ctx: TimelineContext,
    hotkey: HotkeyContext,

    layout: TimelineLayout,
    event_timelines: Query<(), With<EventTimeline>>,

    window_query: Query<&Window>,
    bpm_list: Res<BpmList>,

    mut edits: Edits,

    timeline_pending: Query<&TimelinePending>,
    events: Query<&LineEvent>,

    index: Res<EventIndex>,
    pending: Query<(), With<Pending>>,
) -> Result {
    let Ok(window) = window_query.single() else {
        return Ok(());
    };
    let Some(cursor_position) = window.cursor_position() else {
        return Ok(());
    };

    let Some(item) = layout
        .at(cursor_position.into_egui().to_pos2())
        .filter(|item| event_timelines.contains(item.timeline))
    else {
        return Ok(());
    };
    let Some(line_entity) = item.target else {
        return Ok(());
    };
    let viewport = item.viewport;

    let requested_value = requested_event_value(&hotkey);
    let pending_event = timeline_pending
        .get(item.timeline)
        .ok()
        .and_then(TimelinePending::entity)
        .and_then(|entity| Some((events.get(entity).ok()?, entity)));

    let calc_event_attrs = || {
        let time = ctx.y_to_time(cursor_position.y);
        let beat = bpm_list.beat_at(time).value();
        let beat = ctx.settings.attach(beat);

        let track =
            (((cursor_position.x - viewport.min.x) / (viewport.width() / 5.0)).floor() as u8 + 1)
                .min(5);

        (track, beat)
    };

    if let Some((pending_event, entity)) = pending_event {
        let mut pending_event = *pending_event;
        let (track, beat) = calc_event_attrs();
        pending_event.end_beat = beat.max(pending_event.start_beat + ctx.settings.minimum_beat());
        pending_event.kind = LineEventKind::try_from(track).expect("Unknown event track");
        // The preview may be despawned by a cascade applied before this command
        commands.entity(entity).try_insert(pending_event);
    }

    if let Some(value) = requested_value {
        if let Some((pending_event, entity)) = pending_event {
            // inherit event's start & end value from neighbor events
            let mut new_event = *pending_event;
            new_event.value = value;
            let (track, beat) = calc_event_attrs();
            new_event.kind = LineEventKind::try_from(track).expect("Unknown event track");
            new_event.end_beat = beat.max(new_event.start_beat + ctx.settings.minimum_beat());
            let confirmed = |(entity, _): &(Entity, &LineEvent)| !pending.contains(*entity);
            if let Some((_, last_event)) = index
                .until(line_entity, new_event.kind, new_event.start_beat.value())
                .filter(confirmed)
                .find(|(_, event)| event.end_beat <= new_event.start_beat)
            {
                *new_event.value.start_mut() = last_event.value.end();
            }
            if let Some((_, next_event)) = index
                .since(line_entity, new_event.kind, new_event.end_beat.value())
                .find(confirmed)
            {
                *new_event.value.end_mut() = next_event.value.start();
            }
            commands.entity(entity).despawn();
            edits.once(t!("history.create_events", count = 1), move |commands| {
                commands.spawn((new_event, EventOf(line_entity)));
            });
        } else {
            let (track, beat) = calc_event_attrs();
            let kind = LineEventKind::try_from(track).expect("Unknown event track");
            let status = PendingStatus {
                title: t!("tab.timeline.pending.event").into(),
                hints: vec![
                    PendingHint::key(
                        hotkey.label(CreateEventHotkeys::PlaceConstantEvent),
                        t!("tab.timeline.pending.constant"),
                    ),
                    PendingHint::key(
                        hotkey.label(CreateEventHotkeys::PlaceTransitionEvent),
                        t!("tab.timeline.pending.transition"),
                    ),
                ],
            };
            spawn_pending(
                &mut commands,
                item.timeline,
                status,
                (
                    LineEvent {
                        kind,
                        value,
                        start_beat: beat,
                        end_beat: beat + ctx.settings.minimum_beat(),
                    },
                    EventOf(line_entity),
                ),
            );
        }
    }

    Ok(())
}
