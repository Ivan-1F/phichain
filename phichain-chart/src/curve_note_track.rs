use crate::beat;
use crate::easing::Easing;
use crate::note::{Note, NoteKind};
use num::iter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveNoteTrackOptions {
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
