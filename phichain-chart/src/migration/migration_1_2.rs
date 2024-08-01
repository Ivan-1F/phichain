use crate::migration::Migration;
use anyhow::Context;
use serde_json::{json, Value};

/// Migration from format `1` to `2`
///
/// # Changes
///
/// - Added `name` field for each line: this migration uses `Unnamed Line` for all lines as `name`
pub struct Migration1To2;

impl Migration for Migration1To2 {
    fn migrate(old: &Value) -> anyhow::Result<Value> {
        let mut chart = old.clone();
        for line in chart
            .get_mut("lines")
            .context("Failed to get lines")?
            .as_array_mut()
            .context("`lines` is not an array")?
        {
            line["name"] = json!("Unnamed Line");
        }

        chart["format"] = json!(2);

        Ok(chart)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_migration_1_to_2() {
        let old = json!({
          "format": 1,
          "offset": 0.0,
          "bpm_list": [
            {
              "beat": [0, 0, 1],
              "bpm": 120.0,
              "time": 0.0
            }
          ],
          "lines": [
            {
              "notes": [
                {
                  "kind": "Tap",
                  "above": true,
                  "beat": [0, 1, 1],
                  "x": 0.0,
                  "speed": 3.0
                },
                {
                  "kind": "Tap",
                  "above": true,
                  "beat": [1, 0, 1],
                  "x": 337.5,
                  "speed": 1.0
                }
              ],
              "events": [
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
                    "Custom": [
                      0.5,
                      0.0,
                      0.5,
                      1.0
                    ]
                  }
                }
              ]
            }
          ]
        });

        let new = json!({
          "format": 2,
          "offset": 0.0,
          "bpm_list": [
            {
              "beat": [0, 0, 1],
              "bpm": 120.0,
              "time": 0.0
            }
          ],
          "lines": [
            {
              "name": "Unnamed Line",
              "notes": [
                {
                  "kind": "Tap",
                  "above": true,
                  "beat": [0, 1, 1],
                  "x": 0.0,
                  "speed": 3.0
                },
                {
                  "kind": "Tap",
                  "above": true,
                  "beat": [1, 0, 1],
                  "x": 337.5,
                  "speed": 1.0
                }
              ],
              "events": [
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
                    "Custom": [
                      0.5,
                      0.0,
                      0.5,
                      1.0
                    ]
                  }
                }
              ]
            }
          ]
        });

        assert_eq!(Migration1To2::migrate(&old).unwrap(), new);
    }
}
