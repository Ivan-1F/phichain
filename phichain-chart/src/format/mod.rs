use crate::serialization::PhichainChart;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub mod official;
pub mod rpe;

pub trait Format: Serialize + DeserializeOwned {
    #[allow(dead_code)]
    fn into_phichain(self) -> anyhow::Result<PhichainChart>;

    #[allow(dead_code)]
    fn from_phichain(phichain: PhichainChart) -> anyhow::Result<Self>
    where
        Self: Sized;
}
