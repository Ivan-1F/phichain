use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::ops::{AddAssign, SubAssign};
use std::{
    cmp::Ordering,
    ops::{Add, Sub},
};

use num::{CheckedAdd, FromPrimitive, Rational32, Signed, Zero};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tracing::warn;

#[macro_export]
macro_rules! beat {
    ($whole:expr, $numer:expr, $denom:expr) => {
        $crate::beat::Beat::new(
            $whole as i32,
            num::Rational32::new($numer as i32, $denom as i32),
        )
    };
    ($numer:expr, $denom:expr) => {
        $crate::beat::Beat::new(0, num::Rational32::new($numer as i32, $denom as i32))
    };
    ($whole:expr) => {
        $crate::beat::Beat::new($whole as i32, num::Rational32::new(0, 1))
    };
    () => {
        $crate::beat::Beat::new(0, num::Rational32::new(0, 1))
    };
}

/// A beat in the chart
///
/// The [`Beat`] is represented with a fraction and a float
///
/// # The fraction part
///
/// The fraction part consists of a whole part and a ratio part, powered by [num_rational](https://docs.rs/num-rational/latest/num_rational)
///
/// # The float part
///
/// The float part should not be used in most cases, it is designed to prevent denominator overflow on high precision values
///
/// Up to now, the only usage of the float part is dragging edit, and the float part should only have value during dragging.
/// When the dragging stopped, the float part should be merged into the fraction part after attached to beat lines
#[derive(Clone, Copy)]
pub struct Beat(i32, Rational32, f32);

impl Serialize for Beat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // ignore the float part, since it should not has meaning
        if self.2 != 0.0 {
            warn!(
                "Expected float part of a fraction to be 0 during serialization, got {}",
                self.2
            );
        }
        (self.0, self.1.numer(), self.1.denom()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Beat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (whole, numer, denom) = Deserialize::deserialize(deserializer)?;
        Ok(Beat::new(whole, Rational32::new(numer, denom)))
    }
}

impl Hash for Beat {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.reduced().hash(state);
    }
}

impl Beat {
    pub fn reduce(&mut self) {
        let value = Rational32::from_integer(self.0) + self.1;
        self.0 = value.to_integer();
        self.1 = value.fract();
    }

    pub fn reduced(&self) -> Self {
        let mut ret = *self;
        ret.reduce();
        ret
    }
}

impl Debug for Beat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}+{}/{}", self.0, self.1.numer(), self.1.denom())?;

        if self.2 != 0.0 {
            write!(f, "({})", self.2)?;
        }

        Ok(())
    }
}

pub mod utils {
    use crate::beat;
    use crate::beat::Beat;

    /// Attach a beat value to beat lines with a given density
    pub fn attach(value: f32, density: u32) -> Beat {
        let step = 1.0 / density as f32;

        let rounded_value = (value / step).round() * step;

        let integer_part = rounded_value.floor() as i32;
        let fractional_part =
            ((rounded_value - integer_part as f32) * density as f32).round() as i32;

        beat!(integer_part, fractional_part, density)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_attach() {
            assert_eq!(attach(1.333333, 3), beat!(1, 1, 3));
            assert_eq!(attach(1.3, 4), beat!(1, 1, 4));
            assert_eq!(attach(5.8, 2), beat!(6));
        }
    }
}

impl Beat {
    pub const MAX: Self = Beat(i32::MAX, Rational32::ZERO, 0.0);
    pub const MIN: Self = Beat(i32::MIN, Rational32::ZERO, 0.0);

    pub const ZERO: Self = Beat(0, Rational32::ZERO, 0.0);
    pub const ONE: Self = Beat(1, Rational32::ZERO, 0.0);
}

/// The float part (`self.2`) related impl
impl Beat {
    pub fn float(&self) -> f32 {
        self.2
    }

    pub fn float_mut(&mut self) -> &mut f32 {
        &mut self.2
    }
}

impl From<Beat> for f32 {
    fn from(val: Beat) -> Self {
        val.0 as f32 + *val.1.numer() as f32 / *val.1.denom() as f32 + val.2
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
        Self(*value.trunc().numer(), value.fract(), 0.0)
    }
}

impl Beat {
    pub fn new(whole: i32, ratio: Rational32) -> Self {
        Self(whole, ratio, 0.0)
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

    // TODO: implement `Num`, `Neg` and `Signed` for `Beat`
    pub fn abs(&self) -> Self {
        Self(self.0.abs(), self.1.abs(), self.2.abs())
    }
}

impl Zero for Beat {
    fn zero() -> Self {
        Self::ZERO
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero() && self.1.is_zero() && self.2.is_zero()
    }
}

impl Sub for Beat {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2).reduced()
    }
}

impl SubAssign for Beat {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Add for Beat {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2).reduced()
    }
}

impl AddAssign for Beat {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl CheckedAdd for Beat {
    fn checked_add(&self, rhs: &Self) -> Option<Self> {
        Some(Self(
            self.0.checked_add(rhs.0)?,
            self.1.checked_add(&rhs.1)?,
            self.2 + rhs.2, // FIXME
        ))
    }
}

// TODO: implement `Mul`, `Div` for `Beat`

impl PartialEq for Beat {
    fn eq(&self, other: &Self) -> bool {
        let a = self.reduced();
        let b = other.reduced();
        a.0 == b.0 && a.1 == b.1
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
        let a = self.reduced();
        let b = other.reduced();
        match a.0.cmp(&b.0) {
            Ordering::Equal => a.1.cmp(&b.1),
            ord => ord,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq() {
        assert_eq!(beat!(1, 2, 1), beat!(1, 2, 1));
        assert_eq!(beat!(1, 2), beat!(1, 2));
    }

    #[test]
    fn test_not_eq() {
        assert_ne!(beat!(1, 2, 1), beat!(3, 4, 1));
        assert_ne!(beat!(5, 6, 1), beat!(7, 8, 1));
    }

    #[test]
    #[should_panic]
    fn test_with_zero() {
        beat!(0, 0);
        beat!(1, 0);
        beat!(2, 0);
    }

    #[test]
    fn test_with_negative_numbers() {
        assert_eq!(beat!(-1, -2, 1), beat!(-1, -2, 1));
        assert_ne!(beat!(-1, -2, 1), beat!(1, 2, 1));
    }

    #[test]
    fn test_with_large_numbers() {
        assert_eq!(beat!(1000000, 2000000, 1), beat!(1000000, 2000000, 1));
        assert_ne!(beat!(1000000, 2000000, 1), beat!(2000000, 4000000, 1));
    }

    #[test]
    fn test_with_mixed_sign_numbers() {
        assert_eq!(beat!(1, -2, 1), beat!(1, -2, 1));
        assert_ne!(beat!(1, -2, 1), beat!(-1, 2, 1));
    }

    #[test]
    fn test_addition() {
        let beat1 = beat!(1, 2, 1);
        let beat2 = beat!(3, 4, 1);
        let result = beat1 + beat2;
        assert_eq!(result, beat!(4, 6, 1));
    }

    #[test]
    fn test_subtraction() {
        let beat1 = beat!(5, 3, 1);
        let beat2 = beat!(2, 1, 1);
        let result = beat1 - beat2;
        assert_eq!(result, beat!(3, 2, 1));
    }

    #[test]
    fn test_comparison() {
        assert!(beat!(1, 2, 1) < beat!(3, 4, 1));
        assert!(beat!(5, 6, 1) > beat!(3, 4, 1));
        assert_eq!(beat!(7, 8, 1), beat!(7, 8, 1));
    }

    #[test]
    fn test_serialize() {
        let beat = beat!(1, 2, 1);
        let serialized = serde_json::to_string(&beat).unwrap();
        assert_eq!(serialized, "[1,2,1]");
    }

    #[test]
    fn test_deserialize() {
        let serialized = "[1,2,1]";
        let deserialized: Beat = serde_json::from_str(serialized).unwrap();
        assert_eq!(deserialized, beat!(1, 2, 1));
    }

    #[test]
    fn test_reduce() {
        let mut beat = beat!(1, 3, 2);
        beat.reduce();
        assert_eq!(beat, beat!(2, 1, 2));
        // panic!();
    }

    #[test]
    fn test_reduced() {
        let beat = beat!(1, 3, 2);
        let reduced = beat.reduced();
        assert_eq!(reduced, beat!(2, 1, 2));
    }

    #[test]
    fn test_reduced_negative() {
        let beat = beat!(1, -1, 2);
        let reduced = beat.reduced();
        assert_eq!(reduced, beat!(0, 1, 2));
    }

    #[test]
    fn test_from_f32() {
        let beat: Beat = 1.5f32.into();
        assert_eq!(beat, beat!(1, 1, 2));
    }

    #[test]
    fn test_from_rational32() {
        let rational = Rational32::new(3, 2);
        let beat: Beat = rational.into();
        assert_eq!(beat, beat!(1, 1, 2));
    }

    #[test]
    fn test_from_beat_to_f32() {
        let beat = beat!(1, 1, 2);
        let float: f32 = beat.into();
        assert_eq!(float, 1.5);
    }

    #[test]
    fn test_from_beat_to_rational32() {
        let beat = beat!(1, 1, 2);
        let rational: Rational32 = beat.into();
        assert_eq!(rational, Rational32::new(3, 2));
    }
}
