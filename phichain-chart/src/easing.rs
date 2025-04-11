//! Easing for phichain
//!
//! Checkout https://easings.net/ for more details

use bevy::math::{ops, FloatPow};
use bevy::prelude::CubicSegment;
use serde::{Deserialize, Serialize};
use simple_easing::*;
use std::fmt::{Debug, Display, Formatter};
use strum::EnumIter;

/// TODO: this can be replaced with bevy::prelude::EaseFunction and bevy::prelude::FunctionCurve
#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize, EnumIter)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Easing {
    #[default]
    Linear,
    // Sine
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    // Quad
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    // Cubic
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    //Quart
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    // Quint
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    // Expo
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    // Circ
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    // Back
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    // Elastic
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    // Bounce
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,

    Custom(f32, f32, f32, f32),

    Steps(usize),
    Elastic(f32),
}

impl Easing {
    pub fn is_linear(self) -> bool {
        matches!(self, Easing::Linear)
    }

    #[allow(dead_code)]
    pub fn is_in(self) -> bool {
        matches!(
            self,
            Easing::EaseInSine
                | Easing::EaseInQuad
                | Easing::EaseInCubic
                | Easing::EaseInQuart
                | Easing::EaseInQuint
                | Easing::EaseInExpo
                | Easing::EaseInCirc
                | Easing::EaseInBack
                | Easing::EaseInElastic
                | Easing::EaseInBounce
        )
    }

    #[allow(dead_code)]
    pub fn is_out(self) -> bool {
        matches!(
            self,
            Easing::EaseOutSine
                | Easing::EaseOutQuad
                | Easing::EaseOutCubic
                | Easing::EaseOutQuart
                | Easing::EaseOutQuint
                | Easing::EaseOutExpo
                | Easing::EaseOutCirc
                | Easing::EaseOutBack
                | Easing::EaseOutElastic
                | Easing::EaseOutBounce
        )
    }

    pub fn is_in_out(self) -> bool {
        matches!(
            self,
            Easing::EaseInOutSine
                | Easing::EaseInOutQuad
                | Easing::EaseInOutCubic
                | Easing::EaseInOutQuart
                | Easing::EaseInOutQuint
                | Easing::EaseInOutExpo
                | Easing::EaseInOutCirc
                | Easing::EaseInOutBack
                | Easing::EaseInOutElastic
                | Easing::EaseInOutBounce
        )
    }

    pub fn is_custom(self) -> bool {
        matches!(self, Easing::Custom(_, _, _, _))
    }

    #[allow(dead_code)]
    pub fn is_steps(self) -> bool {
        matches!(self, Easing::Steps(_))
    }

    #[allow(dead_code)]
    pub fn is_elastic(self) -> bool {
        matches!(self, Easing::Elastic(_))
    }
}

impl Easing {
    pub fn ease(self, x: f32) -> f32 {
        match self {
            Self::Linear => linear(x),
            Self::EaseInSine => sine_in(x),
            Self::EaseOutSine => sine_out(x),
            Self::EaseInOutSine => sine_in_out(x),
            Self::EaseInQuad => quad_in(x),
            Self::EaseOutQuad => quad_out(x),
            Self::EaseInOutQuad => quad_in_out(x),
            Self::EaseInCubic => cubic_in(x),
            Self::EaseOutCubic => cubic_out(x),
            Self::EaseInOutCubic => cubic_in_out(x),
            Self::EaseInQuart => quart_in(x),
            Self::EaseOutQuart => quart_out(x),
            Self::EaseInOutQuart => quart_in_out(x),
            Self::EaseInQuint => quint_in(x),
            Self::EaseOutQuint => quint_out(x),
            Self::EaseInOutQuint => quint_in_out(x),
            Self::EaseInExpo => expo_in(x),
            Self::EaseOutExpo => expo_out(x),
            Self::EaseInOutExpo => expo_in_out(x),
            Self::EaseInCirc => circ_in(x),
            Self::EaseOutCirc => circ_out(x),
            Self::EaseInOutCirc => circ_in_out(x),
            Self::EaseInBack => back_in(x),
            Self::EaseOutBack => back_out(x),
            Self::EaseInOutBack => back_in_out(x),
            Self::EaseInElastic => elastic_in(x),
            Self::EaseOutElastic => elastic_out(x),
            Self::EaseInOutElastic => elastic_in_out(x),
            Self::EaseInBounce => bounce_in(x),
            Self::EaseOutBounce => bounce_out(x),
            Self::EaseInOutBounce => bounce_in_out(x),

            Self::Custom(x1, y1, x2, y2) => CubicSegment::new_bezier([x1, y1], [x2, y2]).ease(x),

            Self::Steps(num_steps) => (x * num_steps as f32).round() / num_steps.max(1) as f32,
            Self::Elastic(omega) => {
                1.0 - (1.0 - x).squared()
                    * (2.0 * ops::sin(omega * x) / omega + ops::cos(omega * x))
            }
        }
    }
}

impl Display for Easing {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Easing::Custom(_, _, _, _) => write!(f, "Custom"),
            _ => write!(f, "{:?}", self),
        }
    }
}

pub trait Tween: Sized {
    fn ease(x1: Self, x2: Self, t: f32, easing: Easing) -> f32;
    fn ease_to(self, x2: Self, t: f32, easing: Easing) -> f32 {
        Self::ease(self, x2, t, easing)
    }
}

macro_rules! impl_tween_for_primitive {
    ($($t:ty)*) => {
        $(
            impl Tween for $t {
                fn ease(x1: Self, x2: Self, t: f32, easing: Easing) -> f32 {
                    let t = easing.ease(t);
                    t.mul_add(x2 as f32 - x1 as f32, x1 as f32)
                }
            }
        )*
    }
}

impl_tween_for_primitive!(f32 f64 i8 i16 i32 i64 u8 u16 u32 u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        assert_eq!(Easing::Linear.ease(0.5), 0.5);
        assert_eq!(Easing::EaseInOutSine.ease(0.5), 0.5);
    }

    #[test]
    fn test_tween() {
        assert_eq!(0.0.ease_to(1.0, 0.5, Easing::Linear), 0.5);
        assert_eq!(1.0.ease_to(2.0, 0.5, Easing::Linear), 1.5);
    }
}
