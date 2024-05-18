//! Easing for phichain
//!
//! Checkout https://easings.net/ for more details

use log::error;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use simple_easing::*;

pub type EasingFunction = fn(f32) -> f32;

const EASING_MAP: [EasingFunction; 31] = [
    linear,
    sine_in,
    sine_out,
    sine_in_out,
    quad_in,
    quad_out,
    quad_in_out,
    cubic_in,
    cubic_out,
    cubic_in_out,
    quart_in,
    quart_out,
    quart_in_out,
    quint_in,
    quint_out,
    quint_in_out,
    expo_in,
    expo_out,
    expo_in_out,
    circ_in,
    circ_out,
    circ_in_out,
    back_in,
    back_out,
    back_in_out,
    elastic_in,
    elastic_out,
    elastic_in_out,
    bounce_in,
    bounce_out,
    bounce_in_out,
];

#[derive(
    IntoPrimitive,
    TryFromPrimitive,
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
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
}

impl Easing {
    pub fn ease(self, x: f32) -> f32 {
        let id: u8 = self.into();
        EASING_MAP.get(id as usize).map_or_else(
            || {
                error!("Unknown ease type {:?}", self);
                0.0
            },
            |func| func(x),
        )
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
