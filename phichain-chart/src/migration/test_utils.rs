use crate::migration::migrate;
use crate::serialization::PhichainChart;
use serde_json::Value;

/// Verifies that an intermediate migration result can still be migrated to the
/// latest format and accepted by the complete chart parser.
pub(crate) fn assert_can_deserialize_after_migrating_to_latest(chart: &Value) {
    let latest = migrate(chart).unwrap();
    let json = serde_json::to_string(&latest).unwrap();
    PhichainChart::from_json_str(&json).expect("migrated chart should parse into PhichainChart");
}
