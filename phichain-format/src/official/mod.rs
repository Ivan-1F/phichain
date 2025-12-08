use crate::official::from_phichain::{
    phichain_to_official, OfficialOutputError, OfficialOutputOptions,
};
use crate::official::into_phichain::official_to_phichain;
use crate::official::schema::OfficialChart;
use crate::{ChartFormat, CommonOutputOptions};
use phichain_chart::beat;
use phichain_chart::beat::Beat;
use phichain_chart::event::LineEvent;
use phichain_chart::serialization::PhichainChart;
use phichain_compiler::helpers::are_contiguous;
use thiserror::Error;

mod fitting;
pub mod from_phichain;
mod into_phichain;
pub mod schema;

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

#[derive(Debug, Error)]
pub enum OfficialInputError {
    #[error("expected at leat one line")]
    NoLine,
    #[error("unsupported formatVersion, expected 1 or 3, got {0}")]
    UnsupportedFormatVersion(u32),
}

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
