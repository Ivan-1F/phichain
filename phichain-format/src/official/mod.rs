use crate::official::from_phichain::phichain_to_official;
use crate::official::into_phichain::official_to_phichain;
use crate::{ChartFormat, CommonOutputOptions};
use phichain_chart::event::LineEvent;
use phichain_chart::serialization::PhichainChart;
use phichain_compiler::helpers::are_contiguous;

mod errors;
mod from_phichain;
mod into_phichain;
mod options;
mod schema;

pub use errors::{OfficialInputError, OfficialOutputError};
pub use options::{OfficialInputOptions, OfficialOutputOptions};
pub use schema::OfficialChart;

const DEFAULT_EASING_FITTING_EPSILON: f32 = 1e-1;

fn merge_constant_events(events: Vec<LineEvent>) -> Vec<LineEvent> {
    events.into_iter().fold(Vec::new(), |mut merged, event| {
        if let Some(last) = merged.last_mut() {
            if last.value.is_numeric_constant()
                && event.value.is_numeric_constant()
                && are_contiguous(last, &event)
            {
                // extend the previous event instead of adding a new one
                last.end_beat = event.end_beat;
                return merged;
            }
        }
        merged.push(event);
        merged
    })
}

impl ChartFormat for OfficialChart {
    type InputOptions = OfficialInputOptions;
    type InputError = OfficialInputError;

    type OutputOptions = OfficialOutputOptions;
    type OutputError = OfficialOutputError;

    fn to_phichain(self, opts: &Self::InputOptions) -> Result<PhichainChart, Self::InputError> {
        official_to_phichain(self, opts)
    }

    fn from_phichain(
        phichain: PhichainChart,
        opts: &Self::OutputOptions,
    ) -> Result<Self, Self::OutputError> {
        phichain_to_official(phichain, opts)
    }

    fn apply_common_output_options(mut self, common_options: &CommonOutputOptions) -> Self {
        let round = |value: f32| -> f32 {
            let multiplier = 10_f32.powi(common_options.round as i32);
            (value * multiplier).round() / multiplier
        };

        for line in &mut self.lines {
            for event in &mut line.rotate_events {
                event.start = round(event.start);
                event.end = round(event.end);
            }

            for event in &mut line.opacity_events {
                event.start = round(event.start);
                event.end = round(event.end);
            }

            for event in &mut line.speed_events {
                event.value = round(event.value);
            }

            for event in &mut line.move_events {
                event.start_x = round(event.start_x);
                event.end_x = round(event.end_x);
                event.start_y = round(event.start_y);
                event.end_y = round(event.end_y);
            }

            for note in &mut line.notes_above {
                note.x = round(note.x);
            }
            for note in &mut line.notes_below {
                note.x = round(note.x);
            }
        }

        self
    }
}
