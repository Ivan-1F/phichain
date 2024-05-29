use phichain_chart::serialization::PhiChainChart;

pub mod official;
pub mod rpe;

pub trait Format {
    #[allow(dead_code)]
    fn into_phichain(self) -> anyhow::Result<PhiChainChart>;

    #[allow(dead_code)]
    fn from_phichain(phichain: PhiChainChart) -> anyhow::Result<Self>
    where
        Self: Sized;
}
