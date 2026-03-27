use crate::migration::migrate;
use crate::serialization::PhichainChart;
use serde_json::Value;

/// Verifies that an intermediate migration result can still be migrated to the
/// latest format and deserialized by the current `PhichainChart` schema.
pub(crate) fn assert_can_deserialize_after_migrating_to_latest(chart: &Value) {
    let latest = migrate(chart).unwrap();
    serde_json::from_value::<PhichainChart>(latest)
        .expect("migrated chart should deserialize into PhichainChart");
}
