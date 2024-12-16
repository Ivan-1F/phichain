use bevy::prelude::*;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::easing::Easing;

use crate::editing::command::event::CreateEvent;
use crate::editing::command::EditorCommand;
use crate::editing::pending::Pending;
use crate::editing::DoCommandEvent;
use crate::hotkey::{Hotkey, HotkeyContext, HotkeyExt};
use crate::identifier::{Identifier, IntoIdentifier};
use crate::schedule::EditorSet;
use crate::selection::SelectedLine;
use crate::timeline::{TimelineContext, TimelineItem};
use crate::utils::convert::BevyEguiConvert;
use phichain_chart::event::{LineEvent, LineEventBundle, LineEventKind, LineEventValue};

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

pub struct CreateEventPlugin;

impl Plugin for CreateEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (create_event_system, remove_pending_event_on_esc_system).in_set(EditorSet::Edit),
        )
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

    selected_line: Res<SelectedLine>,

    window_query: Query<&Window>,
    bpm_list: Res<BpmList>,

    mut event: EventWriter<DoCommandEvent>,

    mut pending_event_query: Query<(&mut LineEvent, Entity), With<Pending>>,

    event_query: Query<(&LineEvent, &Parent), Without<Pending>>,
) {
    let window = window_query.single();
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let rect = ctx.viewport.0.into_egui();

    for item in &ctx.settings.container.allocate(rect) {
        if let TimelineItem::Event(timeline) = &item.timeline {
            let viewport = item.viewport;
            let line_entity = timeline.line_entity_from_fallback(selected_line.0);

            if !viewport.contains(cursor_position.into_egui().to_pos2()) {
                continue;
            }

            let calc_event_attrs = || {
                let time = ctx.y_to_time(cursor_position.y);
                let beat = bpm_list.beat_at(time).value();
                let beat = ctx.settings.attach(beat);

                let track =
                    ((cursor_position.x - viewport.min.x) / (viewport.width() / 5.0)).ceil() as u8;

                (track, beat)
            };

            if let Ok((mut pending_event, _)) = pending_event_query.get_single_mut() {
                let (track, beat) = calc_event_attrs();
                pending_event.end_beat =
                    beat.max(pending_event.start_beat + ctx.settings.minimum_beat());
                pending_event.kind = LineEventKind::try_from(track).expect("Unknown event track");
            }

            if hotkey.just_pressed(CreateEventHotkeys::PlaceTransitionEvent)
                || hotkey.just_pressed(CreateEventHotkeys::PlaceConstantEvent)
            {
                if let Ok((pending_event, entity)) = pending_event_query.get_single() {
                    // inherit event's start & end value from neighbor events
                    let mut new_event = *pending_event;
                    let mut events = event_query.iter().collect::<Vec<_>>();
                    events.sort_by_key(|x| x.0.start_beat);
                    if let Some(last_event) = events
                        .iter()
                        .filter(|(e, _)| e.kind == pending_event.kind)
                        .filter(|(_, p)| p.get() == line_entity)
                        .take_while(|(e, _)| e.end_beat <= pending_event.start_beat)
                        .map(|x| x.0)
                        .last()
                    {
                        match new_event.value {
                            LineEventValue::Transition { ref mut start, .. } => {
                                *start = last_event.value.end();
                            }
                            LineEventValue::Constant(ref mut value) => {
                                *value = last_event.value.end();
                            }
                        }
                    }
                    events.reverse();
                    if let Some(next_event) = events
                        .iter()
                        .filter(|(e, _)| e.kind == pending_event.kind)
                        .filter(|(_, p)| p.get() == line_entity)
                        .take_while(|(e, _)| e.start_beat >= pending_event.end_beat)
                        .map(|x| x.0)
                        .last()
                    {
                        match new_event.value {
                            LineEventValue::Transition { ref mut end, .. } => {
                                *end = next_event.value.start();
                            }
                            LineEventValue::Constant(ref mut value) => {
                                *value = next_event.value.start();
                            }
                        }
                    }
                    commands.entity(entity).despawn();
                    event.send(DoCommandEvent(EditorCommand::CreateEvent(
                        CreateEvent::new(line_entity, new_event),
                    )));
                } else {
                    let (track, beat) = calc_event_attrs();
                    let kind = LineEventKind::try_from(track).expect("Unknown event track");
                    commands.entity(line_entity).with_children(|parent| {
                        let value = if hotkey.just_pressed(CreateEventHotkeys::PlaceTransitionEvent)
                        {
                            LineEventValue::transition(0.0, 0.0, Easing::Linear)
                        } else {
                            // hotkey.just_pressed(CreateEventHotkeys::PlaceConstantEvent)
                            LineEventValue::constant(0.0)
                        };
                        parent.spawn((
                            LineEventBundle::new(LineEvent {
                                kind,
                                value,
                                start_beat: beat,
                                end_beat: beat + ctx.settings.minimum_beat(),
                            }),
                            Pending,
                        ));
                    });
                }
            }
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
