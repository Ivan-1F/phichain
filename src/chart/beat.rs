use num::{Rational32, FromPrimitive};

#[derive(Clone, Copy, Debug)]
pub struct Beat(pub Rational32);

impl Into<f32> for Beat {
    fn into(self) -> f32 {
        *self.0.numer() as f32 / *self.0.denom() as f32
    }
}

impl Into<Rational32> for Beat {
    fn into(self) -> Rational32 {
        self.0
    }
}

impl From<f32> for Beat {
    fn from(value: f32) -> Self {
        Self(Rational32::from_f32(value).unwrap())
    }
}

impl From<Rational32> for Beat {
    fn from(value: Rational32) -> Self {
        Self(value)
    }
}

impl Beat {
    pub fn beat(&self) -> i32 {
        *self.0.trunc().numer()
    }

    pub fn numer(&self) -> i32 {
        *self.0.fract().numer()
    }

    pub fn denom(&self) -> i32 {
        *self.0.fract().denom()
    }

    pub fn value(&self) -> f32 {
        (*self).into()
    }
}
