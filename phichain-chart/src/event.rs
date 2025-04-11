use crate::easing::{Easing, Tween};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::beat::Beat;
use crate::primitive;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, IntoPrimitive, TryFromPrimitive,
)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum LineEventKind {
    X = 1,
    Y,
    Rotation,
    Opacity,
    Speed,
}

impl LineEventKind {
    pub fn is_x(&self) -> bool {
        matches!(self, LineEventKind::X)
    }

    pub fn is_y(&self) -> bool {
        matches!(self, LineEventKind::Y)
    }

    pub fn is_rotation(&self) -> bool {
        matches!(self, LineEventKind::Rotation)
    }

    pub fn is_opacity(&self) -> bool {
        matches!(self, LineEventKind::Opacity)
    }

    pub fn is_speed(&self) -> bool {
        matches!(self, LineEventKind::Speed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineEventValue {
    Transition {
        start: f32,
        end: f32,
        easing: Easing,
    },
    Constant(f32),
}

impl LineEventValue {
    pub fn transition(start: f32, end: f32, easing: Easing) -> Self {
        Self::Transition { start, end, easing }
    }

    pub fn constant(value: f32) -> Self {
        Self::Constant(value)
    }

    pub fn negated(&self) -> Self {
        match *self {
            LineEventValue::Transition { start, end, easing } => LineEventValue::Transition {
                start: -start,
                end: -end,
                easing,
            },
            LineEventValue::Constant(value) => LineEventValue::Constant(-value),
        }
    }

    pub fn is_transition(&self) -> bool {
        matches!(self, LineEventValue::Transition { .. })
    }

    pub fn is_constant(&self) -> bool {
        matches!(self, LineEventValue::Constant(_))
    }

    pub fn start(&self) -> f32 {
        match self {
            LineEventValue::Transition { start, .. } => *start,
            LineEventValue::Constant(value) => *value,
        }
    }

    pub fn end(&self) -> f32 {
        match self {
            LineEventValue::Transition { end, .. } => *end,
            LineEventValue::Constant(value) => *value,
        }
    }

    pub fn into_constant(self) -> Self {
        match self {
            LineEventValue::Transition { start, .. } => Self::constant(start),
            LineEventValue::Constant(_) => self,
        }
    }

    pub fn into_transition(self) -> Self {
        match self {
            LineEventValue::Transition { .. } => self,
            LineEventValue::Constant(value) => Self::transition(value, value, Easing::Linear),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct LineEvent {
    pub kind: LineEventKind,
    pub start_beat: Beat,
    pub end_beat: Beat,
    pub value: LineEventValue,
}

impl PartialOrd for LineEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.start_beat.partial_cmp(&other.start_beat)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventEvaluationResult {
    /// The event is affecting the line at the given beat
    Affecting(f32),
    /// The given beat is later than the end beat of this event
    ///
    /// If there's no other events after this event,
    /// the value is inherited from the end value of this event
    Inherited { from: Beat, value: f32 },
    /// The given beat is before the start beat of this event
    ///
    /// This event does not have any effect on the line
    Unaffected,
}

impl Eq for EventEvaluationResult {}

impl PartialOrd for EventEvaluationResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// [`Unaffected`] compares as less than any [`Inherited`] or [`Affecting`]
///
/// [`Inherited`] compares as less than [`Affecting`]
///
/// Two [`Affecting`] compare based on their contained values, two [`Inherited`] compare based on their `from` values
///
/// In other words:
///
/// - [`Unaffected`] < [`Inherited`] < [`Affecting`]
/// - Two [`Affecting`] compare based on their contained values, two [`Inherited`] compare based on their `from` values
///
/// ```rust
/// # use phichain_chart::beat;
/// use phichain_chart::event::EventEvaluationResult as R;
/// assert!(R::Unaffected < R::Affecting(10.0));
/// assert!(R::Unaffected < R::Inherited { from: beat!(0), value: 10.0 });
/// assert!(R::Inherited { from: beat!(0), value: 200.0 } < R::Affecting(10.0));
/// assert!(R::Inherited { from: beat!(0), value: 200.0 } < R::Inherited { from: beat!(2), value: 10.0 });
/// assert!(R::Affecting(5.0) < R::Affecting(10.0));
/// ```
///
/// [`Unaffected`]: EventEvaluationResult::Unaffected
/// [`Inherited`]: EventEvaluationResult::Inherited
/// [`Affecting`]: EventEvaluationResult::Affecting
impl Ord for EventEvaluationResult {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (EventEvaluationResult::Unaffected, EventEvaluationResult::Unaffected) => {
                Ordering::Equal
            }
            (EventEvaluationResult::Unaffected, _) => Ordering::Less,
            (_, EventEvaluationResult::Unaffected) => Ordering::Greater,

            (
                EventEvaluationResult::Inherited { from: a, .. },
                EventEvaluationResult::Inherited { from: b, .. },
            ) => a.cmp(b),
            (EventEvaluationResult::Affecting(a), EventEvaluationResult::Affecting(b)) => {
                a.total_cmp(b)
            }

            (EventEvaluationResult::Inherited { .. }, EventEvaluationResult::Affecting(_)) => {
                Ordering::Less
            }
            (EventEvaluationResult::Affecting(_), EventEvaluationResult::Inherited { .. }) => {
                Ordering::Greater
            }
        }
    }
}

impl EventEvaluationResult {
    /// Returns [`Some`] with the contained value if the variant is [`Affecting`] or [`Inherited`], or [`None`] if [`Unaffected`]
    ///
    /// [`Affecting`]: EventEvaluationResult::Affecting
    /// [`Inherited`]: EventEvaluationResult::Inherited
    /// [`Unaffected`]: EventEvaluationResult::Unaffected
    pub fn value(&self) -> Option<f32> {
        match self {
            EventEvaluationResult::Affecting(value) => Some(*value),
            EventEvaluationResult::Inherited { value, .. } => Some(*value),
            EventEvaluationResult::Unaffected => None,
        }
    }
}

impl LineEvent {
    pub fn evaluate(&self, beat: f32) -> EventEvaluationResult {
        let start_beat = self.start_beat.value();
        let end_beat = self.end_beat.value();
        match self.value {
            LineEventValue::Transition { start, end, easing } => {
                if beat >= start_beat && beat <= end_beat {
                    let percent = (beat - start_beat) / (end_beat - start_beat);
                    EventEvaluationResult::Affecting(start.ease_to(end, percent, easing))
                } else if beat > end_beat {
                    EventEvaluationResult::Inherited {
                        from: self.end_beat,
                        value: end,
                    }
                } else {
                    EventEvaluationResult::Unaffected
                }
            }
            LineEventValue::Constant(value) => {
                if beat >= start_beat && beat <= end_beat {
                    EventEvaluationResult::Affecting(value)
                } else if beat > end_beat {
                    EventEvaluationResult::Inherited {
                        from: self.end_beat,
                        value,
                    }
                } else {
                    EventEvaluationResult::Unaffected
                }
            }
        }
    }

    pub fn evaluate_start_no_effect(&self, beat: f32) -> EventEvaluationResult {
        let start_beat = self.start_beat.value();
        let end_beat = self.end_beat.value();
        match self.value {
            LineEventValue::Transition { start, end, easing } => {
                if beat > start_beat && beat <= end_beat {
                    let percent = (beat - start_beat) / (end_beat - start_beat);
                    EventEvaluationResult::Affecting(start.ease_to(end, percent, easing))
                } else if beat > end_beat {
                    EventEvaluationResult::Inherited {
                        from: self.end_beat,
                        value: end,
                    }
                } else {
                    EventEvaluationResult::Unaffected
                }
            }
            LineEventValue::Constant(value) => {
                if beat > start_beat && beat <= end_beat {
                    EventEvaluationResult::Affecting(value)
                } else if beat > end_beat {
                    EventEvaluationResult::Inherited {
                        from: self.end_beat,
                        value,
                    }
                } else {
                    EventEvaluationResult::Unaffected
                }
            }
        }
    }
}

impl From<LineEvent> for primitive::event::LineEvent {
    fn from(event: LineEvent) -> Self {
        match event.value {
            LineEventValue::Transition { start, end, easing } => Self {
                kind: event.kind,
                start_beat: event.start_beat,
                end_beat: event.end_beat,
                start,
                end,
                easing,
            },
            LineEventValue::Constant(value) => Self {
                kind: event.kind,
                start_beat: event.start_beat,
                end_beat: event.end_beat,
                start: value,
                end: value,
                easing: Easing::Linear,
            },
        }
    }
}

impl From<primitive::event::LineEvent> for LineEvent {
    fn from(event: primitive::event::LineEvent) -> Self {
        Self {
            kind: event.kind,
            start_beat: event.start_beat,
            end_beat: event.end_beat,
            value: LineEventValue::transition(event.start, event.end, event.easing),
        }
    }
}

// TODO: types below should be moved to phichain-game

#[cfg(feature = "bevy")]
#[derive(bevy::prelude::Bundle)]
pub struct LineEventBundle {
    event: LineEvent,
}

#[cfg(feature = "bevy")]
impl LineEventBundle {
    pub fn new(event: LineEvent) -> Self {
        Self { event }
    }
}
