use std::{
    cmp::Ordering,
    ops::{Add, Sub},
};

use num::{FromPrimitive, Rational32};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Beat(i32, Rational32);

impl Beat {
    pub fn attach_to_beat_line(&mut self, density: u32) {
        let original_fraction = self.1;
        let target_denominator = density as i32;
        let closest_numerator = (original_fraction.numer() * target_denominator
            + original_fraction.denom() / 2)
            / original_fraction.denom();
        self.1 = Rational32::new(closest_numerator, target_denominator);
    }
}

impl Beat {
    pub const MAX: Self = Beat(i32::MAX, Rational32::ZERO);
    pub const MIN: Self = Beat(i32::MIN, Rational32::ZERO);

    pub const ZERO: Self = Beat(0, Rational32::ZERO);
    pub const ONE: Self = Beat(1, Rational32::ZERO);
}

impl From<Beat> for f32 {
    fn from(val: Beat) -> Self {
        val.0 as f32 + *val.1.numer() as f32 / *val.1.denom() as f32
    }
}

impl From<Beat> for Rational32 {
    fn from(val: Beat) -> Self {
        Rational32::new(val.0 * val.1.denom() + val.1.numer(), val.denom())
    }
}

impl From<f32> for Beat {
    fn from(value: f32) -> Self {
        Self::from(Rational32::from_f32(value).unwrap())
    }
}

impl From<Rational32> for Beat {
    fn from(value: Rational32) -> Self {
        Self(*value.trunc().numer(), value.fract())
    }
}

impl Beat {
    pub fn new(whole: i32, ratio: Rational32) -> Self {
        Self(whole, ratio)
    }

    pub fn beat(&self) -> i32 {
        self.0
    }

    pub fn numer(&self) -> i32 {
        *self.1.numer()
    }

    pub fn denom(&self) -> i32 {
        *self.1.denom()
    }

    pub fn value(&self) -> f32 {
        (*self).into()
    }
}

impl Sub for Beat {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Add for Beat {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl PartialEq for Beat {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl Eq for Beat {}

impl PartialOrd for Beat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Beat {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.cmp(&other.0) {
            Ordering::Equal => self.1.cmp(&other.1),
            ord => ord,
        }
    }
}
