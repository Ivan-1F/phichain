use phichain_chart::beat::Beat;
use phichain_chart::event::LineEvent;

pub trait EventSequence {
    fn evaluate(&self, beat: Beat) -> f32;
    fn evaluate_start_no_effect(&self, beat: Beat) -> f32;
}

impl EventSequence for Vec<&LineEvent> {
    fn evaluate(&self, beat: Beat) -> f32 {
        let mut ret = 0.0;

        for event in self {
            let result = event.evaluate(beat.value());
            if let Some(value) = result.value() {
                ret = value;
            }
        }

        ret
    }

    fn evaluate_start_no_effect(&self, beat: Beat) -> f32 {
        let mut ret = 0.0;

        for event in self {
            let result = event.evaluate_start_no_effect(beat.value());
            if let Some(value) = result.value() {
                ret = value;
            }
        }

        ret
    }
}
