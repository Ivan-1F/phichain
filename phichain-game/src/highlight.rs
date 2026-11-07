use crate::{GameConfig, GameSet};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use phichain_chart::beat::Beat;
use phichain_chart::note::Note;

pub struct HighlightPlugin;

impl Plugin for HighlightPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HighlightedBeat>().add_systems(
            Update,
            (calc_highlighted_beat_system, mark_highlight_system)
                .chain()
                .in_set(GameSet),
        );
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Highlighted;

#[derive(Resource, Default, Debug, Clone)]
pub struct HighlightedBeat(HashMap<Beat, u32>);

fn calc_highlighted_beat_system(
    query: Query<&Note>,
    mut highlighted_beat: ResMut<HighlightedBeat>,
) {
    highlighted_beat.0.clear();
    for note in &query {
        let counter = highlighted_beat.0.entry(note.beat.reduced()).or_insert(0);
        *counter += 1;
    }
}

fn mark_highlight_system(
    mut commands: Commands,
    query: Query<(Entity, &Note)>,
    highlighted_beat: ResMut<HighlightedBeat>,

    settings: Res<GameConfig>,
) {
    for (entity, note) in &query {
        // The note may be despawned before these visual updates are applied.
        let mut entity = commands.entity(entity);
        if highlighted_beat.0.contains_key(&note.beat.reduced())
            && highlighted_beat.0[&note.beat.reduced()] > 1
            && settings.multi_highlight
        {
            entity.try_insert(Highlighted);
        } else {
            entity.try_remove::<Highlighted>();
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
