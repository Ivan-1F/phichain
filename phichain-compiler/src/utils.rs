use phichain_chart::beat::Beat;
use phichain_chart::event::LineEvent;

pub trait EventSequence {
    fn evaluate(&self, beat: Beat) -> f32;
    fn evaluate_start_no_effect(&self, beat: Beat) -> f32;

    fn x(&self) -> Self;
    fn y(&self) -> Self;
    fn rotation(&self) -> Self;
    #[allow(dead_code)]
    fn opacity(&self) -> Self;
    #[allow(dead_code)]
    fn speed(&self) -> Self;
}

impl EventSequence for Vec<LineEvent> {
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

    fn x(&self) -> Self {
        self.iter().filter(|x| x.kind.is_x()).copied().collect()
    }

    fn y(&self) -> Self {
        self.iter().filter(|x| x.kind.is_y()).copied().collect()
    }

    fn rotation(&self) -> Self {
        self.iter()
            .filter(|x| x.kind.is_rotation())
            .copied()
            .collect()
    }

    fn opacity(&self) -> Self {
        self.iter()
            .filter(|x| x.kind.is_opacity())
            .copied()
            .collect()
    }

    fn speed(&self) -> Self {
        self.iter().filter(|x| x.kind.is_speed()).copied().collect()
    }
}
