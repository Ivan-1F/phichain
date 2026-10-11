use crate::migration::{for_each_line_recursive, Migration};
use serde_json::{json, Value};
use uuid::Uuid;

/// Migration from format `6` to `7`
///
/// # Changes
///
/// - Every chart object (line, note, event, curve note track) gains a unique `id` string
/// - Curve note tracks reference notes by their `id` instead of by index into the
///   line's `notes` array
///
/// # Modifications
///
/// - Lines, notes, events and curve note tracks: a fresh `"id": "<uuid>"` is added
/// - Curve note tracks: `"from": 1, "to": 2` (indices) become the id strings of the
///   notes at those positions. Tracks referencing missing notes are dangling and dropped
pub struct Migration6To7;

fn mint_id() -> Value {
    json!(Uuid::now_v7().to_string())
}

fn migrate_line(line: &mut Value) -> anyhow::Result<()> {
    line["id"] = mint_id();

    let mut note_ids = Vec::new();
    if let Some(notes) = line.get_mut("notes").and_then(|v| v.as_array_mut()) {
        for note in notes {
            let id = mint_id();
            note_ids.push(id.clone());
            note["id"] = id;
        }
    }

    if let Some(events) = line.get_mut("events").and_then(|v| v.as_array_mut()) {
        for event in events {
            event["id"] = mint_id();
        }
    }

    if let Some(tracks) = line
        .get_mut("curve_note_tracks")
        .and_then(|v| v.as_array_mut())
    {
        let mut kept = Vec::with_capacity(tracks.len());
        for track in tracks.iter_mut() {
            let from = track.get("from").and_then(|v| v.as_u64());
            let to = track.get("to").and_then(|v| v.as_u64());
            match (from, to) {
                (Some(from), Some(to))
                    if (from as usize) < note_ids.len() && (to as usize) < note_ids.len() =>
                {
                    track["from"] = note_ids[from as usize].clone();
                    track["to"] = note_ids[to as usize].clone();
                    track["id"] = mint_id();
                    kept.push(track.clone());
                }
                _ => {
                    tracing::warn!(
                        "dropping dangling curve note track during migration: {:?}",
                        track
                    );
                }
            }
        }
        *tracks = kept;
    }

    Ok(())
}

impl Migration for Migration6To7 {
    fn migrate(old: &Value) -> anyhow::Result<Value> {
        let mut chart = old.clone();

        for_each_line_recursive(&mut chart, &mut |line| {
            migrate_line(line)?;
            Ok(())
        })?;

        chart["format"] = json!(7);

        Ok(chart)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migration::test_utils::assert_can_deserialize_after_migrating_to_latest;

    fn old_chart() -> Value {
        json!({
            "format": 6,
            "offset": 0.0,
            "bpm_list": [
                { "beat": [0, 0, 1], "bpm": 120.0, "time": 0.0 }
            ],
            "lines": [
                {
                    "name": "parent",
                    "notes": [
                        { "kind": "tap", "above": true, "beat": [0, 1, 1], "x": 0.0, "speed": 1.0 },
                        { "kind": "tap", "above": true, "beat": [1, 1, 1], "x": 100.0, "speed": 1.0 }
                    ],
                    "events": [
                        {
                            "kind": "x",
                            "start_beat": [0, 1, 1],
                            "end_beat": [1, 1, 1],
                            "value": { "type": "constant", "value": 0.0 }
                        }
                    ],
                    "curve_note_tracks": [
                        { "from": 0, "to": 1, "kind": "drag", "density": 16, "curve": { "type": "linear" } },
                        { "from": 0, "to": 5, "kind": "drag", "density": 16, "curve": { "type": "linear" } }
                    ],
                    "children": [
                        {
                            "name": "child",
                            "notes": [],
                            "events": [],
                            "curve_note_tracks": [],
                            "children": []
                        }
                    ]
                }
            ]
        })
    }

    #[test]
    fn test_ids_are_assigned() {
        let chart = Migration6To7::migrate(&old_chart()).unwrap();
        let line = &chart["lines"][0];

        assert!(line["id"].is_string());
        for note in line["notes"].as_array().unwrap() {
            assert!(note["id"].is_string());
        }
        assert!(line["events"][0]["id"].is_string());
        assert!(line["children"][0]["id"].is_string());

        let ids: Vec<&str> = line["notes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|note| note["id"].as_str().unwrap())
            .collect();
        assert_ne!(ids[0], ids[1]);
    }

    #[test]
    fn test_curve_note_track_references_become_ids() {
        let chart = Migration6To7::migrate(&old_chart()).unwrap();
        let line = &chart["lines"][0];
        let tracks = line["curve_note_tracks"].as_array().unwrap();

        // the dangling track (to = 5) is dropped
        assert_eq!(tracks.len(), 1);

        let notes = line["notes"].as_array().unwrap();
        assert_eq!(tracks[0]["from"], notes[0]["id"]);
        assert_eq!(tracks[0]["to"], notes[1]["id"]);
        assert!(tracks[0]["id"].is_string());
    }

    #[test]
    fn test_format_is_set_to_7() {
        let chart = Migration6To7::migrate(&old_chart()).unwrap();
        assert_eq!(chart["format"], json!(7));
    }

    #[test]
    fn test_deserializable_after_migration() {
        assert_can_deserialize_after_migrating_to_latest(&old_chart());
    }
}
