use crate::migration::Migration;
use anyhow::Context;
use serde_json::{json, Value};

/// Migration from format `4` to `5`
///
/// # Changes
///
/// - Added curve note tracks on line-level
///
/// # Modifications
///
/// - Added a empty `curve_note_tracks` array to all lines
pub struct Migration4To5;

impl Migration for Migration4To5 {
    fn migrate(old: &Value) -> anyhow::Result<Value> {
        let mut chart = old.clone();
        for line in chart
            .get_mut("lines")
            .context("Failed to get lines")?
            .as_array_mut()
            .context("`lines` is not an array")?
        {
            line["curve_note_tracks"] = json!([]);
        }

        chart["format"] = json!(5);

        Ok(chart)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_migration_4_to_5() {
        let old = json!({
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
              "children": [],
            }
          ]
        });

        let new = json!({
          "format": 5,
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
              "children": [],
              "curve_note_tracks": [],
            }
          ]
        });

        assert_eq!(Migration4To5::migrate(&old).unwrap(), new);
    }
}
