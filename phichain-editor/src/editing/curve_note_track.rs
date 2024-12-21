use crate::notification::{ToastsExt, ToastsStorage};
use crate::selection::Selected;
use bevy::prelude::*;
use num::iter;
use phichain_chart::beat;
use phichain_chart::easing::Easing;
use phichain_chart::note::{Note, NoteBundle, NoteKind};
use phichain_game::GameSet;

#[derive(Debug, Clone, Component)]
pub struct CurveNoteTrack {
    pub from: Option<Entity>,
    pub to: Option<Entity>,

    pub density: u32,
    pub easing: Easing,
    pub kind: NoteKind,
}

impl CurveNoteTrack {
    pub fn from(entity: Entity) -> Self {
        Self {
            from: Some(entity),
            to: None,

            density: 16,
            easing: Easing::EaseInOutSine,
            kind: NoteKind::Drag,
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

/// Generate a note sequence from a note to another note with a [`CurveNoteTrack`] option
pub fn generate_notes(from: Note, to: Note, options: &CurveNoteTrack) -> Vec<Note> {
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
                1.0 - options.easing.ease(x)
            } else {
                options.easing.ease(x)
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
        if let Some((from, to)) = track.get_entities() {
            if let (Ok(from), Ok(to)) = (note_query.get(from), note_query.get(to)) {
                let notes = generate_notes(*from.0, *to.0, track);

                // if the curve evaluates to zero notes and is not selected,
                // despawn all existing `CurveNote` objects associated with it, and despawn the `CurveNoteTrack` itself
                if notes.is_empty() && selected.is_none() {
                    for (note, note_entity) in &query {
                        if note.0 == entity {
                            commands.entity(note_entity).despawn();
                        }
                    }
                    commands.entity(entity).despawn();

                    toasts.info(t!("tab.inspector.curve_note_track.removed")); // TODO: this should not be under `tab.inspector`

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
    }
}
