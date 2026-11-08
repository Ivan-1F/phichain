use crate::beat::Beat;
use crate::easing::{Easing, Tween};
use crate::id::{EventId, HasId};
#[cfg(feature = "bevy")]
use bevy::ecs::reflect::ReflectComponent;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, IntoPrimitive, TryFromPrimitive,
)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Reflect))]
#[cfg_attr(feature = "bevy", reflect(Clone, PartialEq, Debug))]
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
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Reflect))]
#[cfg_attr(feature = "bevy", reflect(Clone, PartialEq, Debug))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LineEventValue {
    Transition {
        start: f32,
        end: f32,
        easing: Easing,
    },
    Constant {
        value: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Increasing,
    Decreasing,
    Equal,
}

impl LineEventValue {
    pub fn transition(start: f32, end: f32, easing: Easing) -> Self {
        Self::Transition { start, end, easing }
    }

    pub fn constant(value: f32) -> Self {
        Self::Constant { value }
    }

    pub fn negated(&self) -> Self {
        match *self {
            LineEventValue::Transition { start, end, easing } => LineEventValue::Transition {
                start: -start,
                end: -end,
                easing,
            },
            LineEventValue::Constant { value } => LineEventValue::Constant { value: -value },
        }
    }

    pub fn is_transition(&self) -> bool {
        matches!(self, LineEventValue::Transition { .. })
    }

    pub fn is_constant(&self) -> bool {
        matches!(self, LineEventValue::Constant { .. })
    }

    pub fn is_numeric_constant(&self) -> bool {
        match self {
            LineEventValue::Constant { .. } => true,
            LineEventValue::Transition { start, end, .. } if start == end => true,
            _ => false,
        }
    }

    pub fn start(&self) -> f32 {
        match self {
            LineEventValue::Transition { start, .. } => *start,
            LineEventValue::Constant { value } => *value,
        }
    }

    pub fn end(&self) -> f32 {
        match self {
            LineEventValue::Transition { end, .. } => *end,
            LineEventValue::Constant { value } => *value,
        }
    }

    pub fn start_mut(&mut self) -> &mut f32 {
        match self {
            Self::Transition { start, .. } => start,
            Self::Constant { value } => value,
        }
    }

    pub fn end_mut(&mut self) -> &mut f32 {
        match self {
            Self::Transition { end, .. } => end,
            Self::Constant { value } => value,
        }
    }

    pub fn easing_mut(&mut self) -> Option<&mut Easing> {
        match self {
            Self::Transition { easing, .. } => Some(easing),
            Self::Constant { .. } => None,
        }
    }

    pub fn direction(&self) -> Direction {
        let start = self.start();
        let end = self.end();

        if start < end {
            Direction::Increasing
        } else if start > end {
            Direction::Decreasing
        } else {
            Direction::Equal
        }
    }

    pub fn into_constant(self) -> Self {
        match self {
            LineEventValue::Transition { start, .. } => Self::constant(start),
            LineEventValue::Constant { .. } => self,
        }
    }

    pub fn into_transition(self) -> Self {
        match self {
            LineEventValue::Transition { .. } => self,
            LineEventValue::Constant { value } => Self::transition(value, value, Easing::Linear),
        }
    }

    pub fn easing(&self) -> Easing {
        match self {
            LineEventValue::Transition { easing, .. } => *easing,
            LineEventValue::Constant { .. } => Easing::Linear,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
#[cfg_attr(feature = "bevy", component(immutable))]
#[cfg_attr(feature = "bevy", require(EventId))]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Reflect))]
#[cfg_attr(feature = "bevy", reflect(Component, Clone, PartialEq, Debug))]
pub struct LineEvent {
    pub kind: LineEventKind,
    pub start_beat: Beat,
    pub end_beat: Beat,
    pub value: LineEventValue,
}

impl HasId for LineEvent {
    type Id = EventId;
}

impl PartialOrd for LineEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.start_beat.partial_cmp(&other.start_beat)
    }
}

impl LineEvent {
    pub fn duration(&self) -> Beat {
        self.end_beat - self.start_beat
    }

    /// The value at the beat, holding the start value before the event and the end value after it
    pub fn value_at(&self, beat: f32) -> f32 {
        match self.value {
            LineEventValue::Constant { value } => value,
            LineEventValue::Transition { start, end, easing } => {
                let (from, to) = (self.start_beat.value(), self.end_beat.value());
                if beat >= to {
                    end
                } else {
                    start.ease_to(end, ((beat - from) / (to - from)).max(0.0), easing)
                }
            }
        }
    }
}

#[macro_export]
macro_rules! event {
    ($kind:expr, $from:expr => $to:expr, $start_value:expr => $end_value:expr, $easing:expr $(,)?) => {
        $crate::event::LineEvent {
            kind: $kind,
            start_beat: $from,
            end_beat: $to,
            value: $crate::event::LineEventValue::transition(
                $start_value as f32,
                $end_value as f32,
                $easing,
            ),
        }
    };
    ($kind:expr, $from:expr => $to:expr, $start_value:expr => $end_value:expr $(,)?) => {
        event!($kind, $from => $to, $start_value => $end_value, $crate::easing::Easing::Linear)
    };
    ($kind:expr, $from:expr => $to:expr, $value:expr $(,)?) => {
        $crate::event::LineEvent {
            kind: $kind,
            start_beat: $from,
            end_beat: $to,
            value: $crate::event::LineEventValue::constant($value as f32),
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::beat;
    use crate::easing::Easing;
    use crate::event::LineEventKind;

    #[test]
    fn value_at_follows_the_curve_and_holds_both_ends() {
        let event = event!(LineEventKind::X, beat!(2) => beat!(4), 10 => 30, Easing::EaseInQuad);
        assert_eq!(event.value_at(1.0), 10.0);
        assert_eq!(event.value_at(2.0), 10.0);
        assert_eq!(event.value_at(3.0), 15.0);
        assert_eq!(event.value_at(4.0), 30.0);
        assert_eq!(event.value_at(5.0), 30.0);
    }
}
