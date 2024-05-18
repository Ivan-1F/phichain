use crate::serialization::PhiChainChart;

mod official;

pub trait Format {
    #[allow(dead_code)]
    fn into_phichain(self) -> anyhow::Result<PhiChainChart>;

    #[allow(dead_code)]
    fn from_phichain(phichain: PhiChainChart) -> anyhow::Result<Self>
    where
        Self: Sized;
}
