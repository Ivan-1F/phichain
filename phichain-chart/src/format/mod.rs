use crate::serialization::PhiChainChart;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub mod official;
pub mod rpe;

pub trait Format: Serialize + DeserializeOwned {
    #[allow(dead_code)]
    fn into_phichain(self) -> anyhow::Result<PhiChainChart>;

    #[allow(dead_code)]
    fn from_phichain(phichain: PhiChainChart) -> anyhow::Result<Self>
    where
        Self: Sized;
}
