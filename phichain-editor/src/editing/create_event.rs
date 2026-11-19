use bevy::prelude::*;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::easing::Easing;

use crate::editing::history::Edits;
use crate::hotkey::{Hotkey, HotkeyContext, HotkeyExt};
use crate::identifier::{Identifier, IntoIdentifier};
use crate::notification::{ToastsExt, ToastsStorage};
use crate::schedule::EditorSet;
use crate::timeline::container::TimelineLayout;
use crate::timeline::event::EventTimeline;
use crate::timeline::pending::{spawn_pending, PendingOf};
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

    pending_event: Option<Single<(Entity, &LineEvent, &PendingOf)>>,

    index: Res<EventIndex>,
    mut toasts: ResMut<ToastsStorage>,
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
    let pending_event = pending_event.map(Single::into_inner);
    // a new pending event replaces the one on another timeline
    let replaced = pending_event.map_or(Entity::PLACEHOLDER, |(entity, ..)| entity);
    let pending_event = pending_event
        .filter(|(.., owner)| owner.0 == item.timeline)
        .map(|(entity, event, _)| (event, entity));

    let calc_event_attrs = || {
        let time = ctx.y_to_time(cursor_position.y);
        let beat = bpm_list.beat_at(time).value();
        let beat = ctx.settings.attach(beat);

        let track =
            (((cursor_position.x - viewport.min.x) / (viewport.width() / 5.0)).floor() as u8 + 1)
                .min(5);

        (track, beat)
    };

    let placement_value = |event: &LineEvent, excluded: Entity| {
        let confirmed = |(other, _): &(Entity, &LineEvent)| *other != excluded;
        let start = index
            .until(line_entity, event.kind, event.start_beat.value())
            .find(confirmed)
            .map_or_else(
                || event.kind.default_value(),
                |(_, event)| event.value.end(),
            );
        match event.value {
            LineEventValue::Constant { .. } => LineEventValue::constant(start),
            LineEventValue::Transition { easing, .. } => {
                let end = index
                    .since(line_entity, event.kind, event.end_beat.value())
                    .find(confirmed)
                    .map_or(0.0, |(_, event)| event.value.start());
                LineEventValue::transition(start, end, easing)
            }
        }
    };

    if let Some((pending_event, entity)) = pending_event {
        // pending events follow the cursor, staying where they last fit
        let mut event = *pending_event;
        let (track, beat) = calc_event_attrs();
        event.end_beat = beat.max(event.start_beat + ctx.settings.minimum_beat());
        event.kind = LineEventKind::try_from(track).expect("Unknown event track");
        if !index.fits(line_entity, entity, &event) {
            event = *pending_event;
        }
        event.value = requested_value.unwrap_or(event.value);
        event.value = placement_value(&event, entity);

        if requested_value.is_some() {
            commands.entity(entity).despawn();
            edits.once(t!("history.create_events", count = 1), move |commands| {
                commands.spawn((event, EventOf(line_entity)));
            });
        } else if event != *pending_event {
            // The preview may be despawned by a cascade applied before this command
            commands.entity(entity).try_insert(event);
        }
    } else if let Some(value) = requested_value {
        let (track, beat) = calc_event_attrs();
        let mut event = LineEvent {
            kind: LineEventKind::try_from(track).expect("Unknown event track"),
            value,
            start_beat: beat,
            end_beat: beat + ctx.settings.minimum_beat(),
        };
        if !index.fits(line_entity, replaced, &event) {
            toasts.error(t!("history.events_overlap"));
            return Ok(());
        }
        event.value = placement_value(&event, replaced);
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
            (event, EventOf(line_entity)),
        );
    }

    Ok(())
}
