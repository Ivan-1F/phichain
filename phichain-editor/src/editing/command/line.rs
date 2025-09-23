use crate::events::line::{DespawnLineEvent, SpawnLineEvent};
use crate::events::EditorEvent;
use crate::timing::ChartTime;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::event::{EventEvaluationResult, LineEvent, LineEventKind, LineEventValue};
use phichain_chart::serialization::SerializedLine;
use phichain_game::event::Events;
use phichain_game::serialization::{SerializeLine, SerializeLineParam};
use undo::Edit;

#[derive(Debug, Copy, Clone)]
pub struct CreateLine(Option<Entity>);

impl CreateLine {
    pub fn new() -> Self {
        Self(None)
    }
    pub fn with_target(target: Entity) -> Self {
        Self(Some(target))
    }
}

impl Edit for CreateLine {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        let entity = SpawnLineEvent::builder()
            .line(SerializedLine::default())
            .maybe_target(self.0)
            .build()
            .run(target);
        self.0 = Some(entity);
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some(entity) = self.0 {
            DespawnLineEvent::builder()
                .target(entity)
                .keep_entity(true)
                .build()
                .run(target);
        }
    }
}

#[derive(Debug, Clone)]
pub struct RemoveLine {
    entity: Entity,
    line: Option<(SerializedLine, Option<Entity>)>,
}

impl RemoveLine {
    pub fn new(entity: Entity) -> Self {
        Self { entity, line: None }
    }
}

impl Edit for RemoveLine {
    type Target = World;
    type Output = ();

    // To persist entity ID for each line, we do not despawn the line entity directly
    // Instead, we retain the entity, despawn all its children and remove all components
    // When undoing, we restore the line entity and its children
    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        let entity = self.entity;
        let parent = target
            .entity(self.entity)
            .get::<ChildOf>()
            .map(|x| x.parent());

        let serialized_line = target
            .run_system_once(move |line_params: SerializeLineParam| {
                SerializedLine::serialize_line(&line_params, entity)
            })
            .expect("Failed to serialize line");

        self.line = Some((serialized_line, parent));
        DespawnLineEvent::builder()
            .target(self.entity)
            .keep_entity(true)
            .build()
            .run(target);
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some(ref line) = self.line {
            // restore line entity and its children
            SpawnLineEvent::builder()
                .line(line.0.clone())
                .maybe_parent(line.1)
                .target(self.entity)
                .build()
                .run(target);
        }
    }
}

/// Move a line as child of another line
#[derive(Debug, Clone)]
pub struct MoveLineAsChild {
    entity: Entity,
    prev_parent: Option<Entity>,
    /// Some = move as child of this line, None = move to root
    target: Option<Entity>,
}

impl MoveLineAsChild {
    pub fn new(entity: Entity, target: Option<Entity>) -> Self {
        Self {
            entity,
            prev_parent: None,
            target,
        }
    }
}

impl Edit for MoveLineAsChild {
    type Target = World;
    type Output = ();

    fn edit(&mut self, world: &mut Self::Target) -> Self::Output {
        self.prev_parent = world
            .entity(self.entity)
            .get::<ChildOf>()
            .map(|x| x.parent());
        match self.target {
            None => {
                world.entity_mut(self.entity).remove::<ChildOf>();
            }
            Some(target) => {
                world.entity_mut(self.entity).insert(ChildOf(target));
            }
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        target.entity_mut(self.entity).remove::<ChildOf>();
        if let Some(prev_parent) = self.prev_parent {
            target.entity_mut(self.entity).insert(ChildOf(prev_parent));
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreateLineFromSelected {
    created_entity: Option<Entity>,
    selected_line: Entity,
}

impl CreateLineFromSelected {
    pub fn new(selected_line: Entity) -> Self {
        Self {
            created_entity: None,
            selected_line,
        }
    }
}

impl Edit for CreateLineFromSelected {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        // Get current state values by evaluating events at current time
        let selected_line = self.selected_line;
        let current_state = target
            .run_system_once(
                move |event_query: Query<&LineEvent>,
                      events_query: Query<&Events>,
                      time: Res<ChartTime>,
                      bpm_list: Res<BpmList>| {
                    let beat: f32 = bpm_list.beat_at(time.0).into();

                    let mut x_value = EventEvaluationResult::Unaffected;
                    let mut y_value = EventEvaluationResult::Unaffected;
                    let mut rotation_value = EventEvaluationResult::Unaffected;
                    let mut opacity_value = EventEvaluationResult::Unaffected;
                    let mut speed_value = EventEvaluationResult::Unaffected;

                    if let Ok(events) = events_query.get(selected_line) {
                        for event in events.iter().filter_map(|x| event_query.get(x).ok()) {
                            let value = event.evaluate(beat);
                            match event.kind {
                                LineEventKind::X => x_value = x_value.max(value),
                                LineEventKind::Y => y_value = y_value.max(value),
                                LineEventKind::Rotation => {
                                    rotation_value = rotation_value.max(value)
                                }
                                LineEventKind::Opacity => opacity_value = opacity_value.max(value),
                                LineEventKind::Speed => speed_value = speed_value.max(value),
                            }
                        }
                    }

                    (
                        x_value.value().unwrap_or(0.0),
                        y_value.value().unwrap_or(0.0),
                        rotation_value.value().unwrap_or(0.0),
                        opacity_value.value().unwrap_or(0.0),
                        speed_value.value().unwrap_or(10.0),
                    )
                },
            )
            .expect("Failed to get current line state");

        let new_line = SerializedLine {
            line: Default::default(),
            notes: vec![],
            events: vec![
                LineEvent {
                    kind: LineEventKind::X,
                    value: LineEventValue::constant(current_state.0),
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                },
                LineEvent {
                    kind: LineEventKind::Y,
                    value: LineEventValue::constant(current_state.1),
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                },
                LineEvent {
                    kind: LineEventKind::Rotation,
                    value: LineEventValue::constant(current_state.2),
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                },
                LineEvent {
                    kind: LineEventKind::Opacity,
                    value: LineEventValue::constant(current_state.3),
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                },
                LineEvent {
                    kind: LineEventKind::Speed,
                    value: LineEventValue::constant(current_state.4),
                    start_beat: Beat::ZERO,
                    end_beat: Beat::ONE,
                },
            ],
            children: vec![],
            curve_note_tracks: vec![],
        };

        let entity = SpawnLineEvent::builder().line(new_line).build().run(target);

        self.created_entity = Some(entity);
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some(entity) = self.created_entity {
            DespawnLineEvent::builder()
                .target(entity)
                .keep_entity(true)
                .build()
                .run(target);
        }
    }
}
