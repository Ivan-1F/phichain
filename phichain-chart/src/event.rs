use crate::easing::{Easing, Tween};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::beat::Beat;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, IntoPrimitive, TryFromPrimitive,
)]
#[repr(u8)]
pub enum LineEventKind {
    X = 1,
    Y,
    Rotation,
    Opacity,
    Speed,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
pub struct LineEvent {
    pub kind: LineEventKind,
    pub start: f32,
    pub end: f32,
    pub start_beat: Beat,
    pub end_beat: Beat,

    pub easing: Easing,
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
    Inherited(f32),
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
/// Two [`Affecting`] or [`Inherited`] compare based on their contained values
///
/// In other words:
///
/// - [`Unaffected`] < [`Inherited`] < [`Affecting`]
/// - [`Inherited`] and [`Affecting`] compare based on their contained values
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

            (EventEvaluationResult::Inherited(a), EventEvaluationResult::Inherited(b)) => {
                a.total_cmp(b)
            }
            (EventEvaluationResult::Affecting(a), EventEvaluationResult::Affecting(b)) => {
                a.total_cmp(b)
            }

            (EventEvaluationResult::Inherited(_), EventEvaluationResult::Affecting(_)) => {
                Ordering::Less
            }
            (EventEvaluationResult::Affecting(_), EventEvaluationResult::Inherited(_)) => {
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
            EventEvaluationResult::Inherited(value) => Some(*value),
            EventEvaluationResult::Unaffected => None,
        }
    }
}

impl LineEvent {
    pub fn evaluate(&self, beat: f32) -> EventEvaluationResult {
        let start_beat = self.start_beat.value();
        let end_beat = self.end_beat.value();
        if beat >= start_beat && beat <= end_beat {
            let percent = (beat - start_beat) / (end_beat - start_beat);
            EventEvaluationResult::Affecting(self.start.ease_to(self.end, percent, self.easing))
        } else if beat > end_beat {
            EventEvaluationResult::Inherited(self.end)
        } else {
            EventEvaluationResult::Unaffected
        }
    }

    pub fn evaluate_start_no_effect(&self, beat: f32) -> EventEvaluationResult {
        let start_beat = self.start_beat.value();
        let end_beat = self.end_beat.value();
        if beat > start_beat && beat <= end_beat {
            let percent = (beat - start_beat) / (end_beat - start_beat);
            EventEvaluationResult::Affecting(self.start.ease_to(self.end, percent, self.easing))
        } else if beat > end_beat {
            EventEvaluationResult::Inherited(self.end)
        } else {
            EventEvaluationResult::Unaffected
        }
    }
}

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
