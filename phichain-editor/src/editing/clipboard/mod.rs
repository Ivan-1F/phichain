use crate::action::ActionRegistrationExt;
use crate::editing::history::{ChangeSet, Edits};
use crate::hotkey::modifier::Modifier;
use crate::hotkey::Hotkey;
use crate::selection::{Selected, SelectedLine};
use crate::timeline::TimelineContext;
use crate::utils::convert::BevyEguiConvert;
use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::ecs::reflect::ReflectComponent;
use bevy::ecs::relationship::RelationshipHookMode;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy::scene::DynamicSceneBuilder;
use phichain_chart::beat::Beat;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::curve_note_track::CurveNoteTrackOptions;
use phichain_chart::event::LineEvent;
use phichain_chart::note::Note;
use phichain_game::curve_note_track::{
    CurveNoteTrackFrom, CurveNoteTrackTo, CurveNoteTracksFrom, CurveNoteTracksTo,
};
use phichain_game::event::EventOf;
use phichain_game::{Derived, Pending};
use std::sync::Arc;

#[derive(Resource, Default)]
struct EditorClipboard {
    scene: Arc<DynamicScene>,
    first_beat: Option<Beat>,
}

pub struct ClipboardPlugin;

impl Plugin for ClipboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorClipboard>()
            .add_action(
                "phichain.copy",
                copy_system,
                Some(Hotkey::new(KeyCode::KeyC, vec![Modifier::Control])),
            )
            .add_action(
                "phichain.cut",
                cut_system,
                Some(Hotkey::new(KeyCode::KeyX, vec![Modifier::Control])),
            )
            .add_action(
                "phichain.paste",
                paste_system,
                Some(Hotkey::new(KeyCode::KeyV, vec![Modifier::Control])),
            );
    }
}

fn copy_system(world: &mut World) -> Result {
    let mut entities: EntityHashSet = world
        .query_filtered::<Entity, (With<Selected>, Without<Derived>, Without<Pending>)>()
        .iter(world)
        .filter(|entity| {
            world.get::<Note>(*entity).is_some()
                || world.get::<LineEvent>(*entity).is_some()
                || world.get::<CurveNoteTrackTo>(*entity).is_some()
        })
        .collect();
    let track_endpoints: Vec<(Entity, Entity)> = world
        .query_filtered::<
            (&CurveNoteTrackFrom, &CurveNoteTrackTo),
            (With<Selected>, Without<Pending>),
        >()
        .iter(world)
        .map(|(from, to)| (from.0, to.0))
        .collect();
    for (from, to) in track_endpoints {
        entities.extend([from, to]);
    }
    let first_beat = entities
        .iter()
        .filter_map(|entity| {
            world
                .get::<Note>(*entity)
                .map(|note| note.beat)
                .or_else(|| {
                    world
                        .get::<LineEvent>(*entity)
                        .map(|event| event.start_beat)
                })
        })
        .min();
    let scene = DynamicSceneBuilder::from_world(world)
        .deny_all()
        .allow_component::<Note>()
        .allow_component::<LineEvent>()
        .allow_component::<CurveNoteTrackOptions>()
        .allow_component::<CurveNoteTrackFrom>()
        .allow_component::<CurveNoteTrackTo>()
        .extract_entities(entities.into_iter())
        .build();
    *world.resource_mut::<EditorClipboard>() = EditorClipboard {
        scene: Arc::new(scene),
        first_beat,
    };
    Ok(())
}

fn cut_system(world: &mut World) -> Result {
    copy_system(world)?;
    let targets: Vec<Entity> = world
        .query_filtered::<Entity, (With<Selected>, Without<Derived>, Without<Pending>)>()
        .iter(world)
        .filter(|entity| {
            world.get::<Note>(*entity).is_some()
                || world.get::<LineEvent>(*entity).is_some()
                || world.get::<CurveNoteTrackTo>(*entity).is_some()
        })
        .collect();
    let endpoints: Vec<Entity> = world
        .query_filtered::<
            (&CurveNoteTrackFrom, &CurveNoteTrackTo),
            (With<Selected>, Without<Pending>),
        >()
        .iter(world)
        .flat_map(|(from, to)| [from.0, to.0])
        .collect();
    world
        .run_system_once_with(
            |In((targets, endpoints)): In<(Vec<Entity>, Vec<Entity>)>, mut edits: Edits| {
                edits.once(
                    |changes: &ChangeSet| {
                        t!("history.cut", objects = changes.destroyed_counts().text()).into_owned()
                    },
                    move |commands| {
                        for entity in targets {
                            commands.entity(entity).try_despawn();
                        }
                        // Endpoints that anchored nothing besides the cut tracks die
                        // with them; shared endpoints stay and paste clones them
                        commands.queue(move |world: &mut World| {
                            for endpoint in endpoints {
                                let Ok(entity) = world.get_entity_mut(endpoint) else {
                                    continue;
                                };
                                let orphaned = entity.get::<CurveNoteTracksFrom>().is_none()
                                    && entity.get::<CurveNoteTracksTo>().is_none();
                                if orphaned {
                                    entity.despawn();
                                }
                            }
                        });
                    },
                );
            },
            (targets, endpoints),
        )
        .expect("Failed to cut selection");
    Ok(())
}

fn paste_system(
    clipboard: Res<EditorClipboard>,
    window: Single<&Window>,
    selected_line: Res<SelectedLine>,
    ctx: TimelineContext,
    bpm_list: Res<BpmList>,
    mut edits: Edits,
) -> Result {
    let (Some(first), Some(cursor)) = (clipboard.first_beat, window.cursor_position()) else {
        return Ok(());
    };
    if !ctx.viewport.0.contains(cursor) {
        return Ok(());
    }
    let timeline = ctx
        .settings
        .container
        .allocate(ctx.viewport.0.into_egui())
        .into_iter()
        .find(|item| item.viewport.x_range().contains(cursor.x));
    let Some(timeline) = timeline else {
        return Ok(());
    };
    let line = timeline.timeline.line_entity().unwrap_or(selected_line.0);
    let beat = ctx
        .settings
        .attach(bpm_list.beat_at(ctx.y_to_time(cursor.y)).value());
    let delta = beat - first;
    let scene = clipboard.scene.clone();
    edits.once(
        |changes: &ChangeSet| {
            t!("history.paste", objects = changes.created_counts().text()).into_owned()
        },
        move |commands| {
            commands.queue(move |world: &mut World| paste(world, &scene, line, delta));
        },
    );
    Ok(())
}

fn paste(world: &mut World, scene: &DynamicScene, line: Entity, delta: Beat) {
    let mut entities: EntityHashMap<Entity> = scene
        .entities
        .iter()
        .map(|entity| (entity.entity, world.spawn_empty().id()))
        .collect();
    let registry = world.resource::<AppTypeRegistry>().clone();
    let registry = registry.read();
    // Only source components are copied. Run Bevy's relationship hooks so the
    // inverse collections refer to the copies, and let required components mint IDs.
    for source in &scene.entities {
        let target = entities[&source.entity];
        for value in &source.components {
            let ty = value.get_represented_type_info().unwrap().type_id();
            let component = registry.get_type_data::<ReflectComponent>(ty).unwrap();
            component.apply_or_insert_mapped(
                &mut world.entity_mut(target),
                value.as_partial_reflect(),
                &registry,
                &mut entities,
                RelationshipHookMode::Run,
            );
        }
    }
    for entity in entities.values().copied() {
        if let Some(note) = world.get::<Note>(entity).copied() {
            world.entity_mut(entity).insert((
                Note {
                    beat: note.beat + delta,
                    ..note
                },
                ChildOf(line),
            ));
        } else if let Some(event) = world.get::<LineEvent>(entity).copied() {
            world.entity_mut(entity).insert((
                LineEvent {
                    start_beat: event.start_beat + delta,
                    end_beat: event.end_beat + delta,
                    ..event
                },
                EventOf(line),
            ));
        } else {
            world.entity_mut(entity).insert(ChildOf(line));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editing::history::{open_document, EditorHistory, HistoryPlugin};
    use crate::id_index::{IdIndex, IdIndexPlugin};
    use phichain_chart::event::{LineEventKind, LineEventValue};
    use phichain_chart::id::NoteId;
    use phichain_chart::line::Line;
    use phichain_chart::note::NoteKind;

    fn fixture() -> App {
        let mut app = App::new();
        app.add_plugins((IdIndexPlugin, HistoryPlugin))
            .init_resource::<EditorClipboard>();
        app
    }

    fn spawn_note(world: &mut World, line: Entity, beat: f32, selected: bool) -> Entity {
        let note = Note::new(NoteKind::Tap, true, Beat::from(beat), 0.0, 1.0);
        if selected {
            world.spawn((note, ChildOf(line), Selected)).id()
        } else {
            world.spawn((note, ChildOf(line))).id()
        }
    }

    fn spawn_track(
        world: &mut World,
        line: Entity,
        from: Entity,
        to: Entity,
        selected: bool,
    ) -> Entity {
        let track = (
            CurveNoteTrackFrom(from),
            CurveNoteTrackTo(to),
            ChildOf(line),
        );
        if selected {
            world.spawn((track, Selected)).id()
        } else {
            world.spawn(track).id()
        }
    }

    fn run_paste(world: &mut World, scene: Arc<DynamicScene>, line: Entity, delta: Beat) {
        world
            .run_system_once_with(
                |In((scene, line, delta)): In<(Arc<DynamicScene>, Entity, Beat)>,
                 mut edits: Edits| {
                    edits.once(
                        |changes: &ChangeSet| {
                            t!("history.paste", objects = changes.created_counts().text())
                                .into_owned()
                        },
                        move |commands| {
                            commands
                                .queue(move |world: &mut World| paste(world, &scene, line, delta));
                        },
                    );
                },
                (scene, line, delta),
            )
            .unwrap();
    }

    #[test]
    fn cutting_endpoints_does_not_carry_their_track() {
        let mut app = fixture();
        let world = app.world_mut();
        let line = world.spawn(Line::default()).id();
        let from = spawn_note(world, line, 1.0, true);
        let to = spawn_note(world, line, 4.0, true);
        let old_ids: Vec<_> = [from, to]
            .into_iter()
            .map(|entity| *world.get::<NoteId>(entity).unwrap())
            .collect();
        spawn_track(world, line, from, to, false);
        world.spawn((
            LineEvent {
                kind: LineEventKind::X,
                start_beat: Beat::ONE,
                end_beat: Beat::from(4.0),
                value: LineEventValue::constant(5.0),
            },
            EventOf(line),
            Selected,
        ));
        open_document(world);
        cut_system(world).unwrap();
        assert_eq!(world.query::<&Note>().iter(world).count(), 0);
        assert_eq!(world.query::<&CurveNoteTrackTo>().iter(world).count(), 0);
        let clipboard = world.resource::<EditorClipboard>().scene.clone();
        assert_eq!(clipboard.entities.len(), 3);
        assert_eq!(
            world.resource::<EditorHistory>().undo_label().as_deref(),
            Some("Cut 2 notes, 1 event, 1 curve track")
        );
        for delta in [Beat::from(8.0), Beat::from(16.0)] {
            run_paste(world, clipboard.clone(), line, delta);
        }
        let ids: Vec<_> = world.query::<&NoteId>().iter(world).copied().collect();
        assert_eq!(ids.len(), 4);
        assert!(ids.iter().all(|id| !old_ids.contains(id)));
        assert_eq!(world.query::<&CurveNoteTrackTo>().iter(world).count(), 0);
        world.resource_scope(|world, mut history: Mut<EditorHistory>| history.undo(world));
        assert_eq!(world.query::<&Note>().iter(world).count(), 2);
        world.resource_scope(|world, mut history: Mut<EditorHistory>| history.redo(world));
        assert_eq!(world.query::<&Note>().iter(world).count(), 4);
        assert!(ids
            .iter()
            .all(|id| world.resource::<IdIndex>().entity(id.uuid()).is_some()));

        // a selected track captures its endpoints
        let notes: Vec<Entity> = world
            .query_filtered::<Entity, With<Note>>()
            .iter(world)
            .take(2)
            .collect();
        world
            .run_system_once_with(
                |In((line, from, to)): In<(Entity, Entity, Entity)>, mut edits: Edits| {
                    edits.once("create track", move |commands| {
                        commands.spawn((
                            CurveNoteTrackFrom(from),
                            CurveNoteTrackTo(to),
                            ChildOf(line),
                            Selected,
                        ));
                    });
                },
                (line, notes[0], notes[1]),
            )
            .unwrap();
        copy_system(world).unwrap();
        let clipboard = world.resource::<EditorClipboard>().scene.clone();
        run_paste(world, clipboard, line, Beat::from(32.0));
        assert_eq!(
            world.resource::<EditorHistory>().undo_label().as_deref(),
            Some("Paste 2 notes, 1 curve track")
        );
    }

    #[test]
    fn cutting_a_track_moves_the_exclusive_endpoint_and_keeps_the_shared_one() {
        let mut app = fixture();
        let world = app.world_mut();
        let line = world.spawn(Line::default()).id();
        let a = spawn_note(world, line, 1.0, false);
        let b = spawn_note(world, line, 4.0, false);
        let c = spawn_note(world, line, 8.0, false);
        let ab = spawn_track(world, line, a, b, true);
        let bc = spawn_track(world, line, b, c, false);
        open_document(world);
        cut_system(world).unwrap();
        assert!(world.get_entity(a).is_err());
        assert!(world.get_entity(ab).is_err());
        assert!(world.get_entity(b).is_ok());
        assert!(world.get_entity(bc).is_ok());
        assert_eq!(
            world.resource::<EditorHistory>().undo_label().as_deref(),
            Some("Cut 1 note, 1 curve track")
        );
        let clipboard = world.resource::<EditorClipboard>().scene.clone();
        run_paste(world, clipboard, line, Beat::from(8.0));
        assert_eq!(
            world.resource::<EditorHistory>().undo_label().as_deref(),
            Some("Paste 2 notes, 1 curve track")
        );
        assert_eq!(world.query::<&Note>().iter(world).count(), 4);
        let tracks: Vec<(Entity, Entity)> = world
            .query::<(&CurveNoteTrackFrom, &CurveNoteTrackTo)>()
            .iter(world)
            .map(|(from, to)| (from.0, to.0))
            .collect();
        assert_eq!(tracks.len(), 2);
        assert!(tracks.contains(&(b, c)));
        let (from, to) = tracks.into_iter().find(|pair| *pair != (b, c)).unwrap();
        assert!(from != b && to != b);
        assert_eq!(world.get::<Note>(from).unwrap().beat, Beat::from(9.0));
        assert_eq!(world.get::<Note>(to).unwrap().beat, Beat::from(12.0));
        world.resource_scope(|world, mut history: Mut<EditorHistory>| history.undo(world));
        world.resource_scope(|world, mut history: Mut<EditorHistory>| history.undo(world));
        assert_eq!(world.query::<&Note>().iter(world).count(), 3);
        assert_eq!(world.query::<&CurveNoteTrackTo>().iter(world).count(), 2);
    }

    #[test]
    fn cutting_an_endpoint_kills_the_track_but_does_not_carry_it() {
        let mut app = fixture();
        let world = app.world_mut();
        let line = world.spawn(Line::default()).id();
        let a = spawn_note(world, line, 1.0, true);
        let b = spawn_note(world, line, 4.0, false);
        spawn_track(world, line, a, b, false);
        open_document(world);
        cut_system(world).unwrap();
        assert!(world.get_entity(a).is_err());
        assert!(world.get_entity(b).is_ok());
        assert_eq!(world.query::<&CurveNoteTrackTo>().iter(world).count(), 0);
        assert_eq!(
            world.resource::<EditorHistory>().undo_label().as_deref(),
            Some("Cut 1 note, 1 curve track")
        );
        let clipboard = world.resource::<EditorClipboard>().scene.clone();
        run_paste(world, clipboard, line, Beat::from(8.0));
        assert_eq!(
            world.resource::<EditorHistory>().undo_label().as_deref(),
            Some("Paste 1 note")
        );
        assert_eq!(world.query::<&Note>().iter(world).count(), 2);
        assert_eq!(world.query::<&CurveNoteTrackTo>().iter(world).count(), 0);
    }

    #[test]
    fn cutting_a_track_with_two_shared_endpoints_removes_only_the_track() {
        let mut app = fixture();
        let world = app.world_mut();
        let line = world.spawn(Line::default()).id();
        let x = spawn_note(world, line, 0.0, false);
        let a = spawn_note(world, line, 1.0, false);
        let b = spawn_note(world, line, 4.0, false);
        let c = spawn_note(world, line, 8.0, false);
        spawn_track(world, line, x, a, false);
        let ab = spawn_track(world, line, a, b, true);
        spawn_track(world, line, b, c, false);
        open_document(world);
        cut_system(world).unwrap();
        assert!(world.get_entity(ab).is_err());
        assert_eq!(world.query::<&Note>().iter(world).count(), 4);
        assert_eq!(world.query::<&CurveNoteTrackTo>().iter(world).count(), 2);
        assert_eq!(
            world.resource::<EditorHistory>().undo_label().as_deref(),
            Some("Cut 1 curve track")
        );
        let clipboard = world.resource::<EditorClipboard>().scene.clone();
        run_paste(world, clipboard, line, Beat::from(16.0));
        assert_eq!(
            world.resource::<EditorHistory>().undo_label().as_deref(),
            Some("Paste 2 notes, 1 curve track")
        );
    }
}
