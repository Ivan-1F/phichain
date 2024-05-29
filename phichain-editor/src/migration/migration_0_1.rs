use crate::migration::Migration;
use anyhow::{bail, Context};
use serde_json::{json, Value};

/// Migration from format `0` (without `format` field) to `1`
///
/// # Changes
///
/// - Added key `format` to the root object, indicating the chart format version
/// - The nested arrays within `lines` have been replaced with objects containing `notes` and `events` keys
pub struct Migration0To1;

impl Migration for Migration0To1 {
    fn migrate(old: &Value) -> anyhow::Result<Value> {
        let mut chart = old.clone();
        for line in chart
            .get_mut("lines")
            .context("Failed to get lines")?
            .as_array_mut()
            .context("`lines` is not an array")?
        {
            let line_array = line.as_array_mut().context("A line is not an array")?;
            if line_array.len() != 2 {
                bail!("A line should have 2 elements");
            }

            let notes = &line_array[0];
            let events = &line_array[1];
            *line = json!({
                "notes": notes,
                "events": events,
            });
        }

        chart["format"] = json!(1);

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
          "offset": 0.0,
          "bpm_list": [
            {
              "beat": [0, 0, 1],
              "bpm": 120.0,
              "time": 0.0
            }
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
                  "kind": "Tap",
                  "above": true,
                  "beat": [1, 0, 1],
                  "x": 337.5,
                  "speed": 1.0
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
                    "Custom": [
                      0.5,
                      0.0,
                      0.5,
                      1.0
                    ]
                  }
                }
              ]
            ]
          ]
        });

        let new = json!({
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

        assert_eq!(Migration0To1::migrate(&old).unwrap(), new);
    }
}
