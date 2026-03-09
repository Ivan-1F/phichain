use crate::rpe::schema::RpeBeat;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RpeInputError {
    #[error("RpeBeat has non-zero numerator {0} with zero denominator: {1:?}")]
    ZeroDenominator(i32, RpeBeat),
}
