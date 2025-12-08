use crate::official::DEFAULT_EASING_FITTING_EPSILON;
use phichain_chart::beat;
use phichain_chart::beat::Beat;

#[derive(Debug, Clone)]
pub struct OfficialInputOptions {
    /// Enable easing fitting
    pub easing_fitting: bool,
    /// The epsilon used during easing fitting
    pub easing_fitting_epsilon: f32,
    /// For constant events, how long to shrink them to
    pub constant_event_shrink_to: Beat,
}

impl Default for OfficialInputOptions {
    fn default() -> Self {
        Self {
            easing_fitting: true,
            easing_fitting_epsilon: DEFAULT_EASING_FITTING_EPSILON,
            constant_event_shrink_to: beat!(1, 4),
        }
    }
}

pub struct OfficialOutputOptions {
    pub minimum_beat: Beat,
}

impl Default for OfficialOutputOptions {
    fn default() -> Self {
        Self {
            minimum_beat: beat!(1, 32),
        }
    }
}
