use crate::{GameConfig, GameSet};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use phichain_chart::beat::Beat;
use phichain_chart::note::Note;

pub struct HighlightPlugin;

impl Plugin for HighlightPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NotesByBeat>()
            .add_observer(index_note)
            .add_observer(unindex_note)
            .add_systems(Update, mark_highlight_system.in_set(GameSet));
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Highlighted;

/// Notes grouped by beat, with the beats whose highlight may be out of date
#[derive(Resource, Default)]
struct NotesByBeat {
    notes: HashMap<Beat, Vec<Entity>>,
    dirty: HashSet<Beat>,
}

// notes are immutable, so every change goes through these two triggers

fn index_note(trigger: On<Insert, Note>, notes: Query<&Note>, mut index: ResMut<NotesByBeat>) {
    let Ok(note) = notes.get(trigger.entity) else {
        return;
    };
    let beat = note.beat.reduced();
    index.notes.entry(beat).or_default().push(trigger.entity);
    index.dirty.insert(beat);
}

fn unindex_note(trigger: On<Replace, Note>, notes: Query<&Note>, mut index: ResMut<NotesByBeat>) {
    let Ok(note) = notes.get(trigger.entity) else {
        return;
    };
    let beat = note.beat.reduced();
    if let Some(entities) = index.notes.get_mut(&beat) {
        entities.retain(|entity| *entity != trigger.entity);
        if entities.is_empty() {
            index.notes.remove(&beat);
        }
    }
    index.dirty.insert(beat);
}

/// A note is highlighted if multi highlight is on and another note shares its beat
fn mark_highlight_system(
    mut commands: Commands,
    mut index: ResMut<NotesByBeat>,
    highlighted: Query<Has<Highlighted>>,
    settings: Res<GameConfig>,
) {
    let index = &mut *index;
    if settings.is_changed() {
        index.dirty.extend(index.notes.keys().copied());
    }

    for beat in index.dirty.drain() {
        let Some(entities) = index.notes.get(&beat) else {
            continue;
        };
        let highlight = settings.multi_highlight && entities.len() > 1;
        for &entity in entities {
            if highlighted.get(entity).is_ok_and(|has| has != highlight) {
                // The note may be despawned before these visual updates are applied.
                if highlight {
                    commands.entity(entity).try_insert(Highlighted);
                } else {
                    commands.entity(entity).try_remove::<Highlighted>();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::log::tracing;
    use bevy::log::tracing_subscriber::{self, layer::SubscriberExt, Layer};
    use phichain_chart::note::NoteKind;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn highlights_follow_notes_sharing_a_beat() {
        let mut app = App::new();
        app.insert_resource(GameConfig {
            multi_highlight: true,
            ..default()
        })
        .add_plugins(HighlightPlugin);

        let note = |beat: i32| Note::new(NoteKind::Tap, true, Beat::from(beat as f32), 0.0, 1.0);
        let assert_highlights = |app: &mut App| {
            app.update();
            let world = app.world_mut();
            let multi_highlight = world.resource::<GameConfig>().multi_highlight;
            let notes: Vec<_> = world
                .query::<(&Note, Has<Highlighted>)>()
                .iter(world)
                .map(|(note, highlighted)| (note.beat, highlighted))
                .collect();
            for (beat, highlighted) in &notes {
                let shared = notes.iter().filter(|(other, _)| other == beat).count() > 1;
                assert_eq!(*highlighted, multi_highlight && shared, "note at {beat:?}");
            }
        };

        let world = app.world_mut();
        let a = world.spawn(note(1)).id();
        let b = world.spawn(note(1)).id();
        world.spawn(note(2));
        assert_highlights(&mut app);

        // moving away and onto a shared beat
        app.world_mut().entity_mut(a).insert(note(2));
        assert_highlights(&mut app);

        app.world_mut().entity_mut(b).insert(note(2));
        assert_highlights(&mut app);

        app.world_mut().entity_mut(a).despawn();
        assert_highlights(&mut app);

        app.world_mut().resource_mut::<GameConfig>().multi_highlight = false;
        assert_highlights(&mut app);

        app.world_mut().resource_mut::<GameConfig>().multi_highlight = true;
        assert_highlights(&mut app);
    }

    #[test]
    fn despawn_before_deferred_highlight_updates_is_safe() {
        struct CountWarnings(Arc<AtomicUsize>);
        impl<S: tracing::Subscriber> Layer<S> for CountWarnings {
            fn on_event(
                &self,
                event: &tracing::Event<'_>,
                _: tracing_subscriber::layer::Context<'_, S>,
            ) {
                if matches!(
                    *event.metadata().level(),
                    tracing::Level::WARN | tracing::Level::ERROR
                ) {
                    self.0.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        // Bevy's remove command logs warnings directly, bypassing DefaultErrorHandler.
        // Initialize the log bridge; only this thread's deferred flush counts warnings.
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::WARN)
            .with_writer(std::io::sink)
            .try_init();
        let mut app = App::new();
        app.insert_resource(GameConfig {
            multi_highlight: false,
            ..default()
        })
        .add_plugins(HighlightPlugin);
        let entities: Vec<_> = (0..4)
            .map(|_| {
                app.world_mut()
                    .spawn((
                        Note::new(NoteKind::Tap, true, Beat::ONE, 100.0, 1.0),
                        Highlighted,
                    ))
                    .id()
            })
            .collect();

        // Queue visual updates, then delete notes before those updates are applied.
        app.edit_schedule(Update, |schedule| {
            schedule.set_apply_final_deferred(false);
        });
        app.world_mut().run_schedule(Update);
        for entity in &entities {
            assert!(app.world().get::<Highlighted>(*entity).is_some());
        }
        for entity in &entities[..3] {
            app.world_mut().entity_mut(*entity).despawn();
        }

        let warnings = Arc::new(AtomicUsize::new(0));
        tracing::subscriber::with_default(
            tracing_subscriber::registry().with(CountWarnings(warnings.clone())),
            || {
                app.world_mut()
                    .schedule_scope(Update, |world, schedule| schedule.apply_deferred(world));
            },
        );
        assert_eq!(warnings.load(Ordering::Relaxed), 0);
        let survivor = entities[3];
        assert!(app.world().get::<Note>(survivor).is_some());
        assert!(app.world().get::<Highlighted>(survivor).is_none());
    }
}
