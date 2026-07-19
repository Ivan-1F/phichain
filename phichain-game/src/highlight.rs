use crate::{GameConfig, GameSet};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use phichain_chart::beat::Beat;
use phichain_chart::note::Note;

pub struct HighlightPlugin;

impl Plugin for HighlightPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HighlightedBeat>()
            .add_systems(Update, update_highlight_system.in_set(GameSet));
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Highlighted;

/// Incrementally maintained index of note beats for multi-highlight.
///
/// Rather than rebuilding a beat counter over all notes every frame, this
/// consumes the stream of note changes (added / beat-moved / removed) and
/// updates only the affected beat buckets. Highlight state only flips on
/// bucket-size transitions across the 1 <-> 2 boundary, so per-frame cost is
/// O(changed notes x chord size) instead of O(all notes).
#[derive(Resource, Default, Debug)]
pub struct HighlightedBeat {
    /// Number of notes per (reduced) beat.
    count: HashMap<Beat, u32>,
    /// Note entity -> its current (unreduced) beat, used to find the old
    /// bucket when a note moves or is removed.
    forward: HashMap<Entity, Beat>,
    /// (Reduced) beat -> note entities at that beat, used to find the notes
    /// affected by a bucket-size transition.
    reverse: HashMap<Beat, Vec<Entity>>,
    /// Entities currently carrying the [`Highlighted`] component.
    highlighted: HashSet<Entity>,
}

impl HighlightedBeat {
    /// Insert `entity` into the bucket of `beat` (already reduced).
    ///
    /// Boundary transitions: when the bucket reaches 2 notes, all notes at the
    /// beat gain highlight; when it was already lit, only the newcomer does.
    fn insert_into_bucket(
        &mut self,
        entity: Entity,
        beat: Beat,
        multi: bool,
        to_highlight: &mut Vec<Entity>,
    ) {
        let count = self.count.entry(beat).or_insert(0);
        *count += 1;
        let now = *count;
        self.reverse.entry(beat).or_default().push(entity);

        if !multi {
            return;
        }
        if now == 2 {
            for &other in &self.reverse[&beat] {
                if self.highlighted.insert(other) {
                    to_highlight.push(other);
                }
            }
        } else if now > 2 && self.highlighted.insert(entity) {
            to_highlight.push(entity);
        }
    }

    /// Remove `entity` from the bucket of `beat` (already reduced).
    ///
    /// Boundary transition: when the bucket shrinks to a single note, that
    /// note loses its highlight.
    fn remove_from_bucket(&mut self, entity: Entity, beat: Beat, to_unhighlight: &mut Vec<Entity>) {
        let Some(count) = self.count.get_mut(&beat) else {
            return;
        };
        *count -= 1;
        let remaining = *count;
        if remaining == 0 {
            self.count.remove(&beat);
            self.reverse.remove(&beat);
            return;
        }

        let bucket = self.reverse.get_mut(&beat).unwrap();
        bucket.retain(|e| *e != entity);
        if remaining == 1 {
            let last = bucket[0];
            if self.highlighted.remove(&last) {
                to_unhighlight.push(last);
            }
        }
    }
}

/// Keep [`Highlighted`] components in sync with the note set.
///
/// Per frame, this processes only the notes that were added, beat-moved or
/// removed since the last run (plus `multi_highlight` toggles); untouched
/// notes are never visited.
fn update_highlight_system(
    mut commands: Commands,
    changed: Query<(Entity, &Note), Changed<Note>>,
    mut removed: RemovedComponents<Note>,
    mut index: ResMut<HighlightedBeat>,
    settings: Res<GameConfig>,
    mut last_multi_highlight: Local<Option<bool>>,
) {
    let multi = settings.multi_highlight;
    let settings_toggled = *last_multi_highlight != Some(multi);
    *last_multi_highlight = Some(multi);

    let mut to_highlight: Vec<Entity> = Vec::new();
    let mut to_unhighlight: Vec<Entity> = Vec::new();

    // Setting toggled on: light up every bucket with more than one note.
    if settings_toggled && multi {
        let newly_lit: Vec<Entity> = index
            .reverse
            .iter()
            .filter(|(beat, _)| index.count.get(*beat).copied().unwrap_or(0) > 1)
            .flat_map(|(_, entities)| entities.iter().copied())
            .collect();
        for entity in newly_lit {
            if index.highlighted.insert(entity) {
                to_highlight.push(entity);
            }
        }
    }
    // Setting toggled off: remove every highlight.
    if settings_toggled && !multi {
        for entity in index.highlighted.drain() {
            to_unhighlight.push(entity);
        }
    }

    // Removed notes: look up their last known beat and leave that bucket.
    // Processed first so a recycled entity id is re-inserted cleanly below.
    for entity in removed.read() {
        if let Some(old_beat) = index.forward.remove(&entity) {
            index.remove_from_bucket(entity, old_beat.reduced(), &mut to_unhighlight);
            index.highlighted.remove(&entity);
        }
    }

    // Added or modified notes. `Changed` covers both; the forward index tells
    // us whether we have seen this note before.
    for (entity, note) in &changed {
        let new_beat = note.beat;
        match index.forward.get(&entity).copied() {
            None => {
                index.forward.insert(entity, new_beat);
                index.insert_into_bucket(entity, new_beat.reduced(), multi, &mut to_highlight);
            }
            Some(old_beat) if old_beat == new_beat => {
                // Edit that does not affect the beat (position, speed, kind).
            }
            Some(old_beat) => {
                let old_reduced = old_beat.reduced();
                let new_reduced = new_beat.reduced();
                if old_reduced != new_reduced {
                    index.remove_from_bucket(entity, old_reduced, &mut to_unhighlight);
                    index.insert_into_bucket(entity, new_reduced, multi, &mut to_highlight);
                }
                index.forward.insert(entity, new_beat);
            }
        }
    }

    for entity in to_highlight {
        if let Ok(mut entity) = commands.get_entity(entity) {
            entity.try_insert(Highlighted);
        }
    }
    for entity in to_unhighlight {
        if let Ok(mut entity) = commands.get_entity(entity) {
            entity.remove::<Highlighted>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phichain_chart::beat;
    use phichain_chart::note::NoteKind;

    fn test_app() -> App {
        let mut app = App::new();
        app.init_resource::<HighlightedBeat>()
            .insert_resource(GameConfig::default())
            .add_systems(Update, update_highlight_system);
        app
    }

    fn spawn_note(app: &mut App, beat: Beat) -> Entity {
        app.world_mut()
            .spawn(Note::new(NoteKind::Tap, true, beat, 0.0, 1.0))
            .id()
    }

    fn highlighted(app: &App, entity: Entity) -> bool {
        app.world().get::<Highlighted>(entity).is_some()
    }

    #[test]
    fn notes_sharing_beat_are_highlighted() {
        let mut app = test_app();
        let a = spawn_note(&mut app, beat!(1, 0, 1));
        let b = spawn_note(&mut app, beat!(2, 0, 1));
        let c = spawn_note(&mut app, beat!(1, 0, 1));
        app.update();
        assert!(highlighted(&app, a));
        assert!(!highlighted(&app, b));
        assert!(highlighted(&app, c));
    }

    #[test]
    fn equivalent_beats_share_highlight() {
        let mut app = test_app();
        let a = spawn_note(&mut app, beat!(1, 1, 2)); // 1.5
        let b = spawn_note(&mut app, beat!(1, 2, 4)); // also 1.5, unreduced
        app.update();
        assert!(highlighted(&app, a));
        assert!(highlighted(&app, b));
    }

    #[test]
    fn highlight_updates_when_beat_changes() {
        let mut app = test_app();
        let a = spawn_note(&mut app, beat!(1, 0, 1));
        let b = spawn_note(&mut app, beat!(2, 0, 1));
        app.update();
        assert!(!highlighted(&app, a));
        assert!(!highlighted(&app, b));

        app.world_mut().get_mut::<Note>(b).unwrap().beat = beat!(1, 0, 1);
        app.update();
        assert!(highlighted(&app, a));
        assert!(highlighted(&app, b));
    }

    #[test]
    fn highlight_updates_when_note_removed() {
        let mut app = test_app();
        let a = spawn_note(&mut app, beat!(1, 0, 1));
        let b = spawn_note(&mut app, beat!(1, 0, 1));
        app.update();
        assert!(highlighted(&app, a));

        app.world_mut().despawn(b);
        app.update();
        assert!(!highlighted(&app, a));
    }

    #[test]
    fn move_between_beats_updates_both_buckets() {
        let mut app = test_app();
        let a = spawn_note(&mut app, beat!(1, 0, 1));
        let b = spawn_note(&mut app, beat!(1, 0, 1));
        let c = spawn_note(&mut app, beat!(2, 0, 1));
        app.update();
        assert!(highlighted(&app, a));
        assert!(highlighted(&app, b));
        assert!(!highlighted(&app, c));

        // Move b from beat 1 to beat 2: a becomes alone, b joins c.
        app.world_mut().get_mut::<Note>(b).unwrap().beat = beat!(2, 0, 1);
        app.update();
        assert!(!highlighted(&app, a));
        assert!(highlighted(&app, b));
        assert!(highlighted(&app, c));
    }

    #[test]
    fn chord_boundary_transitions() {
        let mut app = test_app();
        let a = spawn_note(&mut app, beat!(1, 0, 1));
        let b = spawn_note(&mut app, beat!(1, 0, 1));
        let c = spawn_note(&mut app, beat!(1, 0, 1));
        app.update();
        assert!(highlighted(&app, a));
        assert!(highlighted(&app, b));
        assert!(highlighted(&app, c));

        // 3 -> 2: still highlighted.
        app.world_mut().despawn(c);
        app.update();
        assert!(highlighted(&app, a));
        assert!(highlighted(&app, b));

        // 2 -> 1: the remaining note loses highlight.
        app.world_mut().despawn(b);
        app.update();
        assert!(!highlighted(&app, a));
    }

    #[test]
    fn respects_multi_highlight_setting() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<GameConfig>()
            .multi_highlight = false;
        let a = spawn_note(&mut app, beat!(1, 0, 1));
        let _ = spawn_note(&mut app, beat!(1, 0, 1));
        app.update();
        assert!(!highlighted(&app, a));
    }

    #[test]
    fn enabling_multi_highlight_lights_existing_chords() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<GameConfig>()
            .multi_highlight = false;
        let a = spawn_note(&mut app, beat!(1, 0, 1));
        let b = spawn_note(&mut app, beat!(1, 0, 1));
        app.update();
        assert!(!highlighted(&app, a));

        app.world_mut()
            .resource_mut::<GameConfig>()
            .multi_highlight = true;
        app.update();
        assert!(highlighted(&app, a));
        assert!(highlighted(&app, b));
    }

    #[test]
    fn idle_frames_do_not_reinsert_highlighted() {
        let mut app = test_app();
        let a = spawn_note(&mut app, beat!(1, 0, 1));
        let _ = spawn_note(&mut app, beat!(1, 0, 1));
        app.update();
        assert!(highlighted(&app, a));

        let tick = app
            .world()
            .entity(a)
            .get_ref::<Highlighted>()
            .unwrap()
            .last_changed();
        app.update();
        app.update();
        let tick_after = app
            .world()
            .entity(a)
            .get_ref::<Highlighted>()
            .unwrap()
            .last_changed();
        assert_eq!(
            tick, tick_after,
            "Highlighted must not be re-inserted on idle frames"
        );
    }
}
