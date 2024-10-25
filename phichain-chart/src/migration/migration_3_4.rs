use crate::migration::Migration;
use anyhow::Context;
use serde_json::{json, Value};

/// Migration from format `3` to `4`
///
/// # Changes
///
/// - Added parent-children hierarchy for lines
///
/// # Modifications
///
/// - Added a empty `children` array to all lines
pub struct Migration3To4;

impl Migration for Migration3To4 {
    fn migrate(old: &Value) -> anyhow::Result<Value> {
        let mut chart = old.clone();
        for line in chart
            .get_mut("lines")
            .context("Failed to get lines")?
            .as_array_mut()
            .context("`lines` is not an array")?
        {
            line["children"] = json!([]);
        }

        chart["format"] = json!(4);

        Ok(chart)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_migration_3_to_4() {
        let old = json!({
          "format": 3,
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
                  "kind": "tap",
                  "above": true,
                  "beat": [0, 1, 1],
                  "x": 0.0,
                  "speed": 3.0
                },
                {
                  "kind": {
                    "hold": {
                      "hold_beat": [1, 0, 1],
                    },
                  },
                  "above": true,
                  "beat": [0, 1, 1],
                  "x": 0.0,
                  "speed": 3.0
                },
                {
                  "kind": "tap",
                  "above": true,
                  "beat": [1, 0, 1],
                  "x": 337.5,
                  "speed": 1.0
                }
              ],
              "events": [
                {
                  "kind": "x",
                  "start_beat": [0, 0, 1],
                  "end_beat": [1, 0, 1],
                  "value": {
                    "transition": {
                      "start": 0.0,
                      "end": 0.0,
                      "easing": "linear",
                    },
                  },
                },
                {
                  "kind": "y",
                  "start_beat": [0, 0, 1],
                  "end_beat": [1, 0, 1],
                  "value": {
                    "transition": {
                      "start": -300.0,
                      "end": -300.0,
                      "easing": {
                        "custom": [
                          0.5,
                          0.0,
                          0.5,
                          1.0
                        ]
                      },
                    },
                  },
                }
              ]
            }
          ]
        });

        let new = json!({
          "format": 4,
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
                  "kind": "tap",
                  "above": true,
                  "beat": [0, 1, 1],
                  "x": 0.0,
                  "speed": 3.0
                },
                {
                  "kind": {
                    "hold": {
                      "hold_beat": [1, 0, 1],
                    },
                  },
                  "above": true,
                  "beat": [0, 1, 1],
                  "x": 0.0,
                  "speed": 3.0
                },
                {
                  "kind": "tap",
                  "above": true,
                  "beat": [1, 0, 1],
                  "x": 337.5,
                  "speed": 1.0
                }
              ],
              "events": [
                {
                  "kind": "x",
                  "start_beat": [0, 0, 1],
                  "end_beat": [1, 0, 1],
                  "value": {
                    "transition": {
                      "start": 0.0,
                      "end": 0.0,
                      "easing": "linear",
                    },
                  },
                },
                {
                  "kind": "y",
                  "start_beat": [0, 0, 1],
                  "end_beat": [1, 0, 1],
                  "value": {
                    "transition": {
                      "start": -300.0,
                      "end": -300.0,
                      "easing": {
                        "custom": [
                          0.5,
                          0.0,
                          0.5,
                          1.0
                        ]
                      },
                    },
                  },
                }
              ],
              "children": []
            }
          ]
        });

        assert_eq!(Migration3To4::migrate(&old).unwrap(), new);
    }
}
