//! Easing for phichain
//!
//! Checkout https://easings.net/ for more details

use serde::{Deserialize, Serialize};
use simple_easing::*;
use std::fmt::{Debug, Display, Formatter};
use strum::EnumIter;

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

            Self::Custom(x1, y1, x2, y2) => BezierTween::new((x1, y1), (x2, y2)).y(x),
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

// https://github.com/gre/bezier-easing
// https://github.com/TeamFlos/phira/blob/main/prpr/src/core/tween.rs

const SAMPLE_TABLE_SIZE: usize = 21;
const SAMPLE_STEP: f32 = 1. / (SAMPLE_TABLE_SIZE - 1) as f32;
const NEWTON_MIN_STEP: f32 = 1e-3;
const NEWTON_ITERATIONS: usize = 4;
const SUBDIVISION_PRECISION: f32 = 1e-7;
const SUBDIVISION_MAX_ITERATION: usize = 10;
const SLOPE_EPS: f32 = 1e-7;

pub struct BezierTween {
    sample_table: [f32; SAMPLE_TABLE_SIZE],
    pub p1: (f32, f32),
    pub p2: (f32, f32),
}

impl BezierTween {
    fn y(&self, x: f32) -> f32 {
        Self::sample(self.p1.1, self.p2.1, self.t_for_x(x))
    }

    #[inline]
    fn coefficients(x1: f32, x2: f32) -> (f32, f32, f32) {
        ((x1 - x2) * 3. + 1., x2 * 3. - x1 * 6., x1 * 3.)
    }

    #[inline]
    fn sample(x1: f32, x2: f32, t: f32) -> f32 {
        let (a, b, c) = Self::coefficients(x1, x2);
        ((a * t + b) * t + c) * t
    }
    #[inline]
    fn slope(x1: f32, x2: f32, t: f32) -> f32 {
        let (a, b, c) = Self::coefficients(x1, x2);
        (a * 3. * t + b * 2.) * t + c
    }

    fn newton_raphson_iterate(x: f32, mut t: f32, x1: f32, x2: f32) -> f32 {
        for _ in 0..NEWTON_ITERATIONS {
            let slope = Self::slope(x1, x2, t);
            if slope <= SLOPE_EPS {
                return t;
            }
            let diff = Self::sample(x1, x2, t) - x;
            t -= diff / slope;
        }
        t
    }

    fn binary_subdivide(x: f32, mut l: f32, mut r: f32, x1: f32, x2: f32) -> f32 {
        let mut t = (l + r) / 2.;
        for _ in 0..SUBDIVISION_MAX_ITERATION {
            let diff = Self::sample(x1, x2, t) - x;
            if diff.abs() <= SUBDIVISION_PRECISION {
                break;
            }
            if diff > 0. {
                r = t;
            } else {
                l = t;
            }
            t = (l + r) / 2.;
        }
        t
    }

    pub fn t_for_x(&self, x: f32) -> f32 {
        if x == 0. || x == 1. {
            return x;
        }
        let id = (x / SAMPLE_STEP) as usize;
        let id = id.min(SAMPLE_TABLE_SIZE - 1);
        let dist =
            (x - self.sample_table[id]) / (self.sample_table[id + 1] - self.sample_table[id]);
        let init_t = SAMPLE_STEP * (id as f32 + dist);
        match Self::slope(self.p1.0, self.p2.0, init_t) {
            y if y <= SLOPE_EPS => init_t,
            y if y >= NEWTON_MIN_STEP => {
                Self::newton_raphson_iterate(x, init_t, self.p1.0, self.p2.0)
            }
            _ => Self::binary_subdivide(
                x,
                SAMPLE_STEP * id as f32,
                SAMPLE_STEP * (id + 1) as f32,
                self.p1.0,
                self.p2.0,
            ),
        }
    }

    pub fn new(p1: (f32, f32), p2: (f32, f32)) -> Self {
        Self {
            sample_table: std::array::from_fn(|i| Self::sample(p1.0, p2.0, i as f32 * SAMPLE_STEP)),
            p1,
            p2,
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
    fn test_custom() {
        assert_eq!(Easing::Custom(0.0, 0.0, 1.0, 1.0).ease(0.5), 0.5);
        assert_eq!(Easing::Custom(0.0, 0.0, 1.0, 1.0).ease(0.1), 0.1);
        assert_eq!(Easing::Custom(0.0, 0.0, 1.0, 1.0).ease(0.9), 0.9);
    }

    #[test]
    fn test_tween() {
        assert_eq!(0.0.ease_to(1.0, 0.5, Easing::Linear), 0.5);
        assert_eq!(1.0.ease_to(2.0, 0.5, Easing::Linear), 1.5);
    }
}
