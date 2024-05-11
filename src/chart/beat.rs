use std::{cmp::Ordering, ops::{Add, Sub}};

use num::{FromPrimitive, Rational32};

#[derive(Clone, Copy, Debug)]
pub struct Beat(i32, Rational32);

impl Beat {
    pub const ZERO: Self = Beat(0, Rational32::ZERO);
    pub const ONE: Self = Beat(1, Rational32::ZERO);
}

impl Into<f32> for Beat {
    fn into(self) -> f32 {
        self.0 as f32 + *self.1.numer() as f32 / *self.1.denom() as f32
    }
}

impl Into<Rational32> for Beat {
    fn into(self) -> Rational32 {
        Rational32::new(self.0 * self.1.denom() + self.1.numer(), self.denom())
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
    pub fn new(whole: i32, ratio: Rational32) -> Self{
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
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.0.partial_cmp(&other.0) {
            Some(core::cmp::Ordering::Equal) => self.1.partial_cmp(&other.1),
            ord => ord,
        }        
    }
}

impl Ord for Beat {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
