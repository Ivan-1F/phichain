mod compile;
pub mod official;
pub mod rpe;

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::convert::Infallible;

use phichain_chart::serialization::{PhichainChart, SerializedLine};

#[derive(Debug, Clone)]
pub struct CommonOutputOptions {
    pub round: u32,
}

impl Default for CommonOutputOptions {
    fn default() -> Self {
        Self { round: 2 }
    }
}

pub trait ChartFormat: Serialize + DeserializeOwned {
    type InputOptions;
    type InputError;

    type OutputOptions;
    type OutputError;

    fn to_phichain(self, opts: &Self::InputOptions) -> Result<PhichainChart, Self::InputError>;

    fn from_phichain(
        phichain: PhichainChart,
        opts: &Self::OutputOptions,
    ) -> Result<Self, Self::OutputError>;

    /// Apply common output options (like rounding) to the chart
    fn apply_common_output_options(self, common_options: &CommonOutputOptions) -> Self;
}

impl ChartFormat for PhichainChart {
    type InputOptions = ();
    type InputError = Infallible;
    type OutputOptions = ();
    type OutputError = Infallible;

    fn to_phichain(self, _: &Self::InputOptions) -> Result<PhichainChart, Self::InputError> {
        Ok(self)
    }

    fn from_phichain(
        phichain: PhichainChart,
        _: &Self::OutputOptions,
    ) -> Result<Self, Self::OutputError> {
        Ok(phichain)
    }

    fn apply_common_output_options(mut self, common_options: &CommonOutputOptions) -> Self {
        use phichain_chart::event::LineEventValue;

        let round = |value: f32| -> f32 {
            let multiplier = 10_f32.powi(common_options.round as i32);
            (value * multiplier).round() / multiplier
        };

        fn round_line<F>(line: &mut SerializedLine, round: &F)
        where
            F: Fn(f32) -> f32,
        {
            for note in &mut line.notes {
                note.x = round(note.x);
            }

            for event in &mut line.events {
                event.value = match event.value {
                    LineEventValue::Constant(v) => LineEventValue::Constant(round(v)),
                    LineEventValue::Transition { start, end, easing } => {
                        LineEventValue::Transition {
                            start: round(start),
                            end: round(end),
                            easing,
                        }
                    }
                };
            }

            for child in &mut line.children {
                round_line(child, round);
            }
        }

        for line in &mut self.lines {
            round_line(line, &round);
        }

        self
    }
}
