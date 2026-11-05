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
use crate::timeline::pending::PendingOf;
use crate::timeline::TimelineContext;
use crate::utils::convert::BevyEguiConvert;
use phichain_chart::event::{LineEvent, LineEventKind, LineEventValue};
use phichain_game::event::EventOf;

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
    keyboard: Res<ButtonInput<KeyCode>>,

    layout: TimelineLayout,
    event_timelines: Query<(), With<EventTimeline>>,

    window_query: Query<&Window>,
    bpm_list: Res<BpmList>,

    mut edits: Edits,

    pending_event_query: Query<(&LineEvent, Entity, Option<&PendingOf>), With<Pending>>,

    event_query: Query<(&LineEvent, &EventOf), Without<Pending>>,
) -> Result {
    // Cancellation takes priority over preview updates and placement, even without a cursor.
    if keyboard.just_pressed(KeyCode::Escape) {
        for (_, entity, _) in &pending_event_query {
            commands.entity(entity).despawn();
        }
        return Ok(());
    }

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

    let calc_event_attrs = || {
        let time = ctx.y_to_time(cursor_position.y);
        let beat = bpm_list.beat_at(time).value();
        let beat = ctx.settings.attach(beat);

        let track =
            (((cursor_position.x - viewport.min.x) / (viewport.width() / 5.0)).floor() as u8 + 1)
                .min(5);

        (track, beat)
    };

    if let Some((pending_event, entity, _)) = pending_event_query
        .single()
        .ok()
        .filter(|(_, _, owner)| owner.is_some_and(|owner| owner.0 == item.timeline))
    {
        let mut pending_event = *pending_event;
        let (track, beat) = calc_event_attrs();
        pending_event.end_beat = beat.max(pending_event.start_beat + ctx.settings.minimum_beat());
        pending_event.kind = LineEventKind::try_from(track).expect("Unknown event track");
        commands.entity(entity).insert(pending_event);
    }

    if let Some(value) = requested_event_value(&hotkey) {
        if let Ok((pending_event, entity, _)) = pending_event_query.single() {
            // inherit event's start & end value from neighbor events
            let mut new_event = *pending_event;
            new_event.value = value;
            let (track, beat) = calc_event_attrs();
            new_event.kind = LineEventKind::try_from(track).expect("Unknown event track");
            new_event.end_beat = beat.max(new_event.start_beat + ctx.settings.minimum_beat());
            let events: Vec<_> = event_query
                .iter()
                .filter(|(event, parent)| {
                    event.kind == new_event.kind && parent.target() == line_entity
                })
                .map(|(event, _)| event)
                .collect();
            if let Some(last_event) = events
                .iter()
                .filter(|event| event.end_beat <= new_event.start_beat)
                .max_by(|a, b| {
                    a.end_beat
                        .cmp(&b.end_beat)
                        .then_with(|| a.value.end().total_cmp(&b.value.end()))
                })
            {
                *new_event.value.start_mut() = last_event.value.end();
            }
            if let Some(next_event) = events
                .iter()
                .filter(|event| event.start_beat >= new_event.end_beat)
                .min_by(|a, b| {
                    a.start_beat
                        .cmp(&b.start_beat)
                        .then_with(|| b.value.start().total_cmp(&a.value.start()))
                })
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
            commands.spawn((
                LineEvent {
                    kind,
                    value,
                    start_beat: beat,
                    end_beat: beat + ctx.settings.minimum_beat(),
                },
                Pending,
                PendingOf(item.timeline),
                EventOf(line_entity),
            ));
        }
    }

    Ok(())
}
