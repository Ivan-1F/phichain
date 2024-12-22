use crate::notification::{ToastsExt, ToastsStorage};
use crate::selection::Selected;
use bevy::prelude::*;
use num::iter;
use phichain_chart::beat;
use phichain_chart::curve_note_track::CurveNoteTrackOptions;
use phichain_chart::note::{Note, NoteBundle};
use phichain_game::GameSet;

#[derive(Debug, Clone, Component)]
pub struct CurveNoteTrack {
    pub from: Option<Entity>,
    pub to: Option<Entity>,

    pub options: CurveNoteTrackOptions,
}

impl CurveNoteTrack {
    pub fn from(entity: Entity) -> Self {
        Self {
            from: Some(entity),
            to: None,

            options: Default::default(),
        }
    }

    pub fn to(&mut self, entity: Entity) {
        self.to = Some(entity);
    }

    /// Return the [`Entity`] of the origin and the destination
    ///
    /// If one of them is missing, return a [`None`], otherwise a [`Some`]
    pub fn get_entities(&self) -> Option<(Entity, Entity)> {
        if let (Some(from), Some(to)) = (self.from, self.to) {
            Some((from, to))
        } else {
            None
        }
    }
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

pub struct CurveNoteTrackPlugin;

impl Plugin for CurveNoteTrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_curve_note_track_system.in_set(GameSet));
    }
}

#[derive(Component)]
struct CurveNoteCache(Vec<Note>);

#[derive(Component)]
pub struct CurveNote(pub Entity);

fn update_curve_note_track_system(
    mut commands: Commands,
    note_query: Query<(&Note, &Parent)>,
    query: Query<(&CurveNote, Entity)>,
    mut track_query: Query<(
        &CurveNoteTrack,
        Option<&mut CurveNoteCache>,
        Option<&Selected>,
        Entity,
    )>,

    mut toasts: ResMut<ToastsStorage>,
) {
    for (track, cache, selected, entity) in &mut track_query {
        let mut despawn_invalid_track = |message: &str| {
            // despawn all existing `CurveNote` objects associated with it, and despawn the `CurveNoteTrack` itself
            for (note, note_entity) in &query {
                if note.0 == entity {
                    commands.entity(note_entity).despawn();
                }
            }
            commands.entity(entity).despawn();

            toasts.info(t!(message));
        };

        let Some((from, to)) = track.get_entities() else {
            // despawn unselected incomplete track
            if selected.is_none() {
                debug!("despawn unselected incomplete CNT");
                despawn_invalid_track("tab.inspector.curve_note_track.removed.incomplete");
            }
            continue;
        };

        let (Ok(from), Ok(to)) = (note_query.get(from), note_query.get(to)) else {
            // despawn invalid track
            debug!("despawn invalid CNT");
            despawn_invalid_track("tab.inspector.curve_note_track.removed.invalid");
            continue;
        };

        let notes = generate_notes(*from.0, *to.0, &track.options);

        if notes.is_empty() && selected.is_none() {
            // despawn unselected empty tracks
            debug!("despawn unselected empty CNT");
            despawn_invalid_track("tab.inspector.curve_note_track.removed.empty");
            continue;
        }

        let update = match cache {
            None => {
                commands
                    .entity(entity)
                    .insert(CurveNoteCache(notes.clone()));
                true
            }
            Some(mut cache) => {
                if cache.0 != notes {
                    cache.0 = notes.clone();
                    true
                } else {
                    false
                }
            }
        };

        if update {
            for (note, note_entity) in &query {
                if note.0 == entity {
                    commands.entity(note_entity).despawn();
                }
            }
            commands.entity(from.1.get()).with_children(|p| {
                for note in notes {
                    p.spawn((NoteBundle::new(note), CurveNote(entity)));
                }
            });
        }
    }
}
