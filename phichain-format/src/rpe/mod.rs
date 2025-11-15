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

    fn from_phichain(
        _: PhichainChart,
        _: &Self::OutputOptions,
        _: &CommonOutputOptions,
    ) -> Result<Self, Self::OutputError> {
        todo!()
    }
}
