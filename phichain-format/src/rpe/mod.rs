use crate::rpe::schema::{rpe_to_phichain, RpeChart, RpeInputOptions};
use crate::{ChartFormat, CommonOutputOptions};
use phichain_chart::serialization::PhichainChart;
use std::convert::Infallible;

pub mod schema;

impl ChartFormat for RpeChart {
    type InputOptions = RpeInputOptions;
    type InputError = Infallible;
    type OutputOptions = ();
    type OutputError = Infallible;

    fn to_phichain(self, opts: &Self::InputOptions) -> Result<PhichainChart, Self::InputError> {
        Ok(rpe_to_phichain(self, opts))
    }

    fn from_phichain(_: PhichainChart, _: &Self::OutputOptions) -> Result<Self, Self::OutputError> {
        todo!()
    }

    fn apply_common_output_options(mut self, common_options: &CommonOutputOptions) -> Self {
        let round = |value: f32| -> f32 {
            let multiplier = 10_f32.powi(common_options.round as i32);
            (value * multiplier).round() / multiplier
        };

        for line in &mut self.judge_line_list {
            for layer in &mut line.event_layers {
                for event in &mut layer.move_x_events {
                    event.start = round(event.start);
                    event.end = round(event.end);
                }

                for event in &mut layer.move_y_events {
                    event.start = round(event.start);
                    event.end = round(event.end);
                }

                for event in &mut layer.rotate_events {
                    event.start = round(event.start);
                    event.end = round(event.end);
                }

                // alpha events: start/end are i32, no rounding needed

                for event in &mut layer.speed_events {
                    event.start = round(event.start);
                    event.end = round(event.end);
                }
            }

            for note in &mut line.notes {
                note.position_x = round(note.position_x);
            }
        }

        self
    }
}
