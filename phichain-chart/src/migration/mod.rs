use crate::migration::migration_0_1::Migration0To1;
use crate::migration::migration_1_2::Migration1To2;
use crate::migration::migration_2_3::Migration2To3;
use crate::migration::migration_3_4::Migration3To4;
use crate::migration::migration_4_5::Migration4To5;
use crate::migration::migration_5_6::Migration5To6;
use anyhow::{bail, Context};
use serde_json::{json, Value};

mod migration_0_1;
mod migration_1_2;
mod migration_2_3;
mod migration_3_4;
mod migration_4_5;
mod migration_5_6;

pub trait Migration {
    fn migrate(old: &Value) -> anyhow::Result<Value>;
}

pub const CURRENT_FORMAT: u64 = 6;

fn get_format(chart: &Value) -> anyhow::Result<u64> {
    let version = chart
        .get("format")
        .unwrap_or(&json!(0))
        .as_u64()
        .context("format field is not a number")?;

    Ok(version)
}

/// Migrate a chart to the latest format
pub fn migrate(chart: &Value) -> anyhow::Result<Value> {
    let format = get_format(chart)?;

    if format == CURRENT_FORMAT {
        return Ok(chart.clone());
    }

    let new_chart = match format {
        0 => Migration0To1::migrate(chart)?,
        1 => Migration1To2::migrate(chart)?,
        2 => Migration2To3::migrate(chart)?,
        3 => Migration3To4::migrate(chart)?,
        4 => Migration4To5::migrate(chart)?,
        5 => Migration5To6::migrate(chart)?,
        _ => bail!("Unsupported chart format {}", format),
    };

    let new_format = get_format(&new_chart)?;

    if new_format < CURRENT_FORMAT {
        migrate(&new_chart)
    } else {
        Ok(new_chart)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::PhichainChart;

    /// Migrate a v0 chart all the way to the latest format and verify
    /// that the result can be deserialized into PhichainChart
    #[test]
    fn test_full_migration_chain_and_deserialize() {
        let v0 = json!({
          "offset": 0.0,
          "bpm_list": [
            { "beat": [0, 0, 1], "bpm": 120.0, "time": 0.0 }
          ],
          "lines": [
            [
              [
                {
                  "kind": "Tap",
                  "above": true,
                  "beat": [0, 1, 1],
                  "x": 0.0,
                  "speed": 3.0
                },
                {
                  "kind": {
                    "Hold": {
                      "hold_beat": [1, 0, 1]
                    }
                  },
                  "above": true,
                  "beat": [0, 1, 1],
                  "x": 0.0,
                  "speed": 3.0
                }
              ],
              [
                {
                  "kind": "X",
                  "start": 0.0,
                  "end": 0.0,
                  "start_beat": [0, 0, 1],
                  "end_beat": [1, 0, 1],
                  "easing": "Linear"
                },
                {
                  "kind": "Y",
                  "start": -300.0,
                  "end": -300.0,
                  "start_beat": [0, 0, 1],
                  "end_beat": [1, 0, 1],
                  "easing": {
                    "Custom": [0.5, 0.0, 0.5, 1.0]
                  }
                }
              ]
            ]
          ]
        });

        let migrated = migrate(&v0).unwrap();

        assert_eq!(
            migrated.get("format").and_then(|v| v.as_u64()),
            Some(CURRENT_FORMAT),
        );

        let chart: PhichainChart = serde_json::from_value(migrated).unwrap();
        assert_eq!(chart.lines.len(), 1);
        assert_eq!(chart.lines[0].notes.len(), 2);
        assert_eq!(chart.lines[0].events.len(), 2);
    }
}
