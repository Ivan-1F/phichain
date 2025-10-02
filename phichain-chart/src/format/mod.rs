use crate::primitive::PrimitiveChart;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub mod official;
pub mod rpe;

pub trait Format: Serialize + DeserializeOwned {
    fn into_primitive(self) -> anyhow::Result<PrimitiveChart>;

    fn from_primitive(phichain: PrimitiveChart) -> anyhow::Result<Self>
    where
        Self: Sized;
}
