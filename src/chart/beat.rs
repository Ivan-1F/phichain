use fraction::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Beat(pub Fraction);

impl Into<f32> for Beat {
    fn into(self) -> f32 {
        self.0.try_into().unwrap()
    }
}

impl Into<Fraction> for Beat {
    fn into(self) -> Fraction {
        self.0
    }
}

impl<T> From<T> for Beat where Fraction: From<T> {
    fn from(value: T) -> Self {
        Self(Fraction::from(value))
    }
}

impl Beat {
    pub fn beat(&self) -> u32 {
        self.0.trunc().try_into().unwrap()
    }

    pub fn numer(&self) -> u32 {
        *self.0.fract().numer().unwrap() as u32
    }

    pub fn denom(&self) -> u32 {
        *self.0.fract().denom().unwrap() as u32
    }

    pub fn value(&self) -> f32 {
        (*self).into()
    }
}
