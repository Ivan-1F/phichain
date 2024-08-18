use crate::migration::Migration;
use anyhow::{bail, Context};
use convert_case::{Case, Casing};
use serde_json::{json, Value};

/// Migration from format `2` to `3`
///
/// # Changes
///
/// - Introduced event value type: transition / constants
/// - Renamed all enum variants using snake_case, affected: `LineEventKind`, `NoteKind` and `Easing`
///
/// # Modifications
///
/// - Converted all events into transition event, removed `start`, `end` and `easing` and added `value` for all events
/// - Renamed all PascalCase enum variants to snake_case
pub struct Migration2To3;

impl Migration for Migration2To3 {
    fn migrate(old: &Value) -> anyhow::Result<Value> {
        let mut chart = old.clone();
        for line in chart
            .get_mut("lines")
            .context("Failed to get lines")?
            .as_array_mut()
            .context("`lines` is not an array")?
        {
            for event in line["events"]
                .as_array_mut()
                .context("`line.events` is not an array")?
            {
                event["kind"] = json!(event["kind"]
                    .as_str()
                    .context("event kind is not string")?
                    .from_case(Case::Pascal)
                    .to_case(Case::Snake));

                let new_easing = match event["easing"] {
                    Value::String(ref s) => {
                        json!(s.from_case(Case::Pascal).to_case(Case::Snake))
                    }
                    Value::Object(_) => {
                        let bezier = &event["easing"]["Custom"];

                        json!({
                            "custom": bezier,
                        })
                    }
                    ref other => {
                        bail!("expected an object or a string as easing, got: {:?}", other)
                    }
                };

                event["value"] = json!({
                    "transition": {
                        "start": event["start"],
                        "end": event["end"],
                        "easing": new_easing,
                    }
                });
                let event = event.as_object_mut().context("event is not an object")?;
                event.remove("start");
                event.remove("end");
                event.remove("easing");
            }

            for note in line["notes"]
                .as_array_mut()
                .context("`line.notes` is not an array")?
            {
                let new_kind = match note["kind"] {
                    Value::String(ref s) => {
                        json!(s.from_case(Case::Pascal).to_case(Case::Snake))
                    }
                    Value::Object(_) => {
                        let hold_beat = &note["kind"]["Hold"]["hold_beat"];

                        json!({
                            "hold": {
                                "hold_beat": hold_beat,
                            },
                        })
                    }
                    ref other => {
                        bail!(
                            "expected an object or a string as note kind, got: {:?}",
                            other
                        )
                    }
                };

                note["kind"] = new_kind;
            }
        }

        chart["format"] = json!(3);

        Ok(chart)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_migration_2_to_3() {
        let old = json!({
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
                  "kind": {
                    "Hold": {
                      "hold_beat": [1, 0, 1],
                    },
                  },
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

        assert_eq!(Migration2To3::migrate(&old).unwrap(), new);
    }
}
