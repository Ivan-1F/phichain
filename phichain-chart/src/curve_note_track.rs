use crate::beat;
use crate::easing::Easing;
use crate::note::{Note, NoteKind};
use num::iter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveNoteTrackOptions {
    #[serde(flatten)]
    pub kind: NoteKind,
    pub density: u32,
    pub curve: Easing,
}

impl Default for CurveNoteTrackOptions {
    fn default() -> Self {
        Self {
            kind: NoteKind::Drag,
            density: 16,
            curve: Easing::Linear,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveNoteTrack {
    pub from: usize,
    pub to: usize,

    #[serde(flatten)]
    pub options: CurveNoteTrackOptions,
}

/// Generate a note sequence from a note to another note with a [`CurveNoteTrackOptions`] option
pub fn generate_notes(from: Note, to: Note, options: &CurveNoteTrackOptions) -> Vec<Note> {
    // make sure from.beat < to.beat
    let (from, to) = if from.beat < to.beat {
        (from, to)
    } else {
        (to, from)
    };

    let mirror = from.x > to.x;

    let beats = iter::range_step(
        from.beat.min(to.beat),
        from.beat.max(to.beat),
        beat!(1, options.density),
    )
    .collect::<Vec<_>>();
    let notes = beats
        .iter()
        .enumerate()
        .map(|(i, beat)| {
            let x = i as f32 / beats.len() as f32;
            let y = if mirror {
                1.0 - options.curve.ease(x)
            } else {
                options.curve.ease(x)
            };

            Note::new(
                options.kind,
                true,
                *beat,
                (from.x - to.x).abs() * y + from.x.min(to.x),
                1.0,
            )
        })
        .skip(1)
        .collect::<Vec<_>>();

    notes
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn test_serialize_curve_note_track_with_hold_kind() {
        let track = CurveNoteTrack {
            from: 1,
            to: 2,
            options: CurveNoteTrackOptions {
                kind: NoteKind::Hold {
                    hold_beat: beat!(0, 1, 2),
                },
                density: 8,
                curve: Easing::Custom {
                    x1: 0.25,
                    y1: 0.1,
                    x2: 0.25,
                    y2: 1.0,
                },
            },
        };

        let value: Value = serde_json::to_value(track).unwrap();

        assert_eq!(
            value,
            json!({
                "from": 1,
                "to": 2,
                "kind": "hold",
                "hold_beat": [0, 1, 2],
                "density": 8,
                "curve": {
                    "type": "custom",
                    "x1": 0.25_f32,
                    "y1": 0.1_f32,
                    "x2": 0.25_f32,
                    "y2": 1.0_f32
                }
            })
        );
    }

    #[test]
    fn test_deserialize_curve_note_track_with_drag_kind() {
        let value = json!({
            "from": 0,
            "to": 1,
            "kind": "drag",
            "density": 16,
            "curve": {
                "type": "linear"
            }
        });

        let track: CurveNoteTrack = serde_json::from_value(value).unwrap();

        assert_eq!(track.from, 0);
        assert_eq!(track.to, 1);
        assert!(matches!(track.options.kind, NoteKind::Drag));
        assert_eq!(track.options.density, 16);
        assert_eq!(track.options.curve, Easing::Linear);
    }
}
