use crate::migration::Migration;
use anyhow::Context;
use serde_json::{json, Value};

/// Migration from format `5` to `6`
///
/// # Changes
///
/// - NoteKind: flattened into Note level with `kind` as plain string and optional `hold_beat`
/// - LineEventValue: switched to internally tagged with `type` field
/// - Easing: switched to internally tagged with `type` field, data variants use named fields
///
/// # Modifications
///
/// - Notes: `"kind": "tap"` unchanged; `"kind": {"hold": {"hold_beat": ...}}` becomes
///   `"kind": "hold", "hold_beat": ...`
/// - Event values: `{"constant": v}` becomes `{"type": "constant", "value": v}`;
///   `{"transition": {...}}` becomes `{"type": "transition", ...}`
/// - Easings: string `"linear"` becomes `{"type": "linear"}`;
///   `{"custom": [a,b,c,d]}` becomes `{"type": "custom", "x1": a, "y1": b, "x2": c, "y2": d}`;
///   `{"steps": n}` becomes `{"type": "steps", "count": n}`;
///   `{"elastic": o}` becomes `{"type": "elastic", "omega": o}`
pub struct Migration5To6;

fn migrate_easing(easing: &Value) -> Value {
    match easing {
        Value::String(s) => json!({"type": s}),
        Value::Object(map) => {
            if let Some(arr) = map.get("custom") {
                if let Some(arr) = arr.as_array() {
                    return json!({
                        "type": "custom",
                        "x1": arr.first().unwrap_or(&json!(0.0)),
                        "y1": arr.get(1).unwrap_or(&json!(0.0)),
                        "x2": arr.get(2).unwrap_or(&json!(0.0)),
                        "y2": arr.get(3).unwrap_or(&json!(0.0)),
                    });
                }
            }
            if let Some(n) = map.get("steps") {
                return json!({"type": "steps", "count": n});
            }
            if let Some(o) = map.get("elastic") {
                return json!({"type": "elastic", "omega": o});
            }
            easing.clone()
        }
        _ => easing.clone(),
    }
}

fn migrate_event_value(value: &Value) -> Value {
    if let Some(map) = value.as_object() {
        if let Some(v) = map.get("constant") {
            return json!({"type": "constant", "value": v});
        }
        if let Some(inner) = map.get("transition") {
            if let Some(inner_map) = inner.as_object() {
                let mut result = json!({"type": "transition"});
                let obj = result.as_object_mut().unwrap();
                for (k, v) in inner_map {
                    if k == "easing" {
                        obj.insert(k.clone(), migrate_easing(v));
                    } else {
                        obj.insert(k.clone(), v.clone());
                    }
                }
                return result;
            }
        }
    }
    value.clone()
}

/// Migrate a NoteKind `kind` field from externally tagged to internally tagged.
/// Works for both notes and curve_note_tracks.
///
/// `{"kind": {"hold": {"hold_beat": [1, 0, 1]}}}` → `{"kind": "hold", "hold_beat": [1, 0, 1]}`
fn migrate_note_kind(obj: &mut Value) {
    if let Some(Value::Object(map)) = obj.get("kind").cloned() {
        if let Some(hold_inner) = map.get("hold") {
            if let Some(hold_beat) = hold_inner.get("hold_beat") {
                obj["kind"] = json!("hold");
                obj["hold_beat"] = hold_beat.clone();
            }
        }
    }
}

fn migrate_line(line: &mut Value) -> anyhow::Result<()> {
    // Migrate notes
    if let Some(notes) = line.get_mut("notes").and_then(|v| v.as_array_mut()) {
        for note in notes {
            migrate_note_kind(note);
        }
    }

    // Migrate events
    if let Some(events) = line.get_mut("events").and_then(|v| v.as_array_mut()) {
        for event in events {
            if let Some(value) = event.get("value").cloned() {
                event["value"] = migrate_event_value(&value);
            }
        }
    }

    // Migrate curve_note_tracks (contains NoteKind in `kind` and Easing in `curve`)
    if let Some(tracks) = line
        .get_mut("curve_note_tracks")
        .and_then(|v| v.as_array_mut())
    {
        for track in tracks {
            migrate_note_kind(track);
            // Migrate the Easing `curve` field
            if let Some(curve) = track.get("curve").cloned() {
                track["curve"] = migrate_easing(&curve);
            }
        }
    }

    // Recurse into children
    if let Some(children) = line.get_mut("children").and_then(|v| v.as_array_mut()) {
        for child in children {
            migrate_line(child)?;
        }
    }

    Ok(())
}

impl Migration for Migration5To6 {
    fn migrate(old: &Value) -> anyhow::Result<Value> {
        let mut chart = old.clone();

        for line in chart
            .get_mut("lines")
            .context("Failed to get lines")?
            .as_array_mut()
            .context("`lines` is not an array")?
        {
            migrate_line(line)?;
        }

        chart["format"] = json!(6);

        Ok(chart)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migration::test_utils::assert_can_deserialize_after_migrating_to_latest;

    fn old_chart() -> Value {
        json!({
            "format": 5,
            "offset": 0.0,
            "bpm_list": [
                { "beat": [0, 0, 1], "bpm": 120.0, "time": 0.0 }
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
                                    "hold_beat": [1, 0, 1]
                                }
                            },
                            "above": true,
                            "beat": [0, 1, 1],
                            "x": 0.0,
                            "speed": 3.0
                        }
                    ],
                    "events": [
                        {
                            "kind": "x",
                            "start_beat": [0, 0, 1],
                            "end_beat": [1, 0, 1],
                            "value": {
                                "constant": 50.0
                            }
                        },
                        {
                            "kind": "y",
                            "start_beat": [0, 0, 1],
                            "end_beat": [1, 0, 1],
                            "value": {
                                "transition": {
                                    "start": -300.0,
                                    "end": 0.0,
                                    "easing": "linear"
                                }
                            }
                        },
                        {
                            "kind": "y",
                            "start_beat": [1, 0, 1],
                            "end_beat": [2, 0, 1],
                            "value": {
                                "transition": {
                                    "start": 0.0,
                                    "end": 100.0,
                                    "easing": {
                                        "custom": [0.5, 0.0, 0.5, 1.0]
                                    }
                                }
                            }
                        },
                        {
                            "kind": "x",
                            "start_beat": [2, 0, 1],
                            "end_beat": [3, 0, 1],
                            "value": {
                                "transition": {
                                    "start": 0.0,
                                    "end": 10.0,
                                    "easing": {
                                        "steps": 4
                                    }
                                }
                            }
                        },
                        {
                            "kind": "x",
                            "start_beat": [3, 0, 1],
                            "end_beat": [4, 0, 1],
                            "value": {
                                "transition": {
                                    "start": 0.0,
                                    "end": 10.0,
                                    "easing": {
                                        "elastic": 20.0
                                    }
                                }
                            }
                        }
                    ],
                    "children": [],
                    "curve_note_tracks": [
                        {
                            "from": 0,
                            "to": 1,
                            "kind": "drag",
                            "density": 16,
                            "curve": "ease_in_sine"
                        },
                        {
                            "from": 1,
                            "to": 2,
                            "kind": {
                                "hold": {
                                    "hold_beat": [0, 1, 2]
                                }
                            },
                            "density": 8,
                            "curve": {
                                "custom": [0.25, 0.1, 0.25, 1.0]
                            }
                        }
                    ]
                }
            ]
        })
    }

    #[test]
    fn test_migration_5_to_6() {
        let old = old_chart();
        let new = json!({
          "format": 6,
          "offset": 0.0,
          "bpm_list": [
            { "beat": [0, 0, 1], "bpm": 120.0, "time": 0.0 }
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
                  "kind": "hold",
                  "hold_beat": [1, 0, 1],
                  "above": true,
                  "beat": [0, 1, 1],
                  "x": 0.0,
                  "speed": 3.0
                }
              ],
              "events": [
                {
                  "kind": "x",
                  "start_beat": [0, 0, 1],
                  "end_beat": [1, 0, 1],
                  "value": {
                    "type": "constant",
                    "value": 50.0
                  }
                },
                {
                  "kind": "y",
                  "start_beat": [0, 0, 1],
                  "end_beat": [1, 0, 1],
                  "value": {
                    "type": "transition",
                    "start": -300.0,
                    "end": 0.0,
                    "easing": { "type": "linear" }
                  }
                },
                {
                  "kind": "y",
                  "start_beat": [1, 0, 1],
                  "end_beat": [2, 0, 1],
                  "value": {
                    "type": "transition",
                    "start": 0.0,
                    "end": 100.0,
                    "easing": { "type": "custom", "x1": 0.5, "y1": 0.0, "x2": 0.5, "y2": 1.0 }
                  }
                },
                {
                  "kind": "x",
                  "start_beat": [2, 0, 1],
                  "end_beat": [3, 0, 1],
                  "value": {
                    "type": "transition",
                    "start": 0.0,
                    "end": 10.0,
                    "easing": { "type": "steps", "count": 4 }
                  }
                },
                {
                  "kind": "x",
                  "start_beat": [3, 0, 1],
                  "end_beat": [4, 0, 1],
                  "value": {
                    "type": "transition",
                    "start": 0.0,
                    "end": 10.0,
                    "easing": { "type": "elastic", "omega": 20.0 }
                  }
                }
              ],
              "children": [],
              "curve_note_tracks": [
                {
                  "from": 0,
                  "to": 1,
                  "kind": "drag",
                  "density": 16,
                  "curve": { "type": "ease_in_sine" }
                },
                {
                  "from": 1,
                  "to": 2,
                  "kind": "hold",
                  "hold_beat": [0, 1, 2],
                  "density": 8,
                  "curve": { "type": "custom", "x1": 0.25, "y1": 0.1, "x2": 0.25, "y2": 1.0 }
                }
              ]
            }
          ]
        });

        assert_eq!(Migration5To6::migrate(&old).unwrap(), new);
    }

    #[test]
    fn test_migration_5_to_6_output_can_reach_latest_and_deserialize() {
        let new = Migration5To6::migrate(&old_chart()).unwrap();
        assert_can_deserialize_after_migrating_to_latest(&new);
    }
}
