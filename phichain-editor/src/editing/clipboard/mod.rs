use crate::action::ActionRegistrationExt;
use crate::editing::description::ObjectCounts;
use crate::editing::history::Edits;
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
use phichain_game::curve_note_track::{CurveNoteTrackFrom, CurveNoteTrackTo};
use phichain_game::event::EventOf;
use phichain_game::{Derived, Pending};
use std::sync::Arc;

#[derive(Resource, Default)]
struct EditorClipboard {
    scene: Arc<DynamicScene>,
    first_beat: Option<Beat>,
    counts: ObjectCounts,
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
            world.get::<Note>(*entity).is_some() || world.get::<LineEvent>(*entity).is_some()
        })
        .collect();
    let tracks: Vec<_> = world
        .query_filtered::<(
            Entity,
            &CurveNoteTrackFrom,
            &CurveNoteTrackTo,
            Has<Selected>,
        ), Without<Pending>>()
        .iter(world)
        .map(|(entity, from, to, selected)| (entity, from.0, to.0, selected))
        .collect();
    for (_, from, to, selected) in &tracks {
        if *selected {
            entities.extend([*from, *to]);
        }
    }
    for (track, from, to, _) in tracks {
        if entities.contains(&from) && entities.contains(&to) {
            entities.insert(track);
        }
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
    let mut counts = ObjectCounts::default();
    let mut kinds = world.query::<(Has<Note>, Has<LineEvent>, Has<CurveNoteTrackTo>)>();
    for entity in &entities {
        let (note, event, track) = kinds.get(world, *entity)?;
        counts.notes += usize::from(note);
        counts.events += usize::from(event);
        counts.tracks += usize::from(track);
    }
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
        counts,
    };
    Ok(())
}

fn cut_system(world: &mut World) -> Result {
    copy_system(world)?;
    let mut targets = Vec::new();
    let mut counts = ObjectCounts::default();
    for (entity, note, event, track) in world
        .query_filtered::<(Entity, Has<Note>, Has<LineEvent>, Has<CurveNoteTrackTo>), With<Selected>>()
        .iter(world)
    {
        targets.push(entity);
        counts.notes += usize::from(note);
        counts.events += usize::from(event);
        counts.tracks += usize::from(track);
    }
    let description = t!("history.cut", objects = counts.text()).into_owned();
    world
        .run_system_once_with(
            |In((targets, description)): In<(Vec<Entity>, String)>, mut edits: Edits| {
                edits.once(description, move |commands| {
                    for entity in targets {
                        commands.entity(entity).try_despawn();
                    }
                });
            },
            (targets, description),
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
        t!("history.paste", objects = clipboard.counts.text()),
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
    use phichain_game::curve_note_track::{CurveNoteTracksFrom, CurveNoteTracksTo};

    #[test]
    fn cut_then_paste_copies_internal_links_and_mints_ids_for_each_copy() {
        let mut app = App::new();
        app.add_plugins((IdIndexPlugin, HistoryPlugin))
            .init_resource::<EditorClipboard>();
        let world = app.world_mut();
        let line = world.spawn(Line::default()).id();
        let note = Note::new(NoteKind::Tap, true, Beat::ONE, 0.0, 1.0);
        let from = world.spawn((note, ChildOf(line), Selected)).id();
        let to = world
            .spawn((
                Note {
                    beat: Beat::from(4.0),
                    ..note
                },
                ChildOf(line),
                Selected,
            ))
            .id();
        let old_ids: Vec<_> = [from, to]
            .into_iter()
            .map(|entity| *world.get::<NoteId>(entity).unwrap())
            .collect();
        world.spawn((
            CurveNoteTrackFrom(from),
            CurveNoteTrackTo(to),
            ChildOf(line),
        ));
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
        assert_eq!(clipboard.entities.len(), 4);
        let counts = &world.resource::<EditorClipboard>().counts;
        assert_eq!((counts.notes, counts.events, counts.tracks), (2, 1, 1));
        for delta in [Beat::from(8.0), Beat::from(16.0)] {
            world
                .run_system_once_with(
                    |In((scene, line, delta)): In<(Arc<DynamicScene>, Entity, Beat)>,
                     mut edits: Edits| {
                        edits.once("paste", move |commands| {
                            commands
                                .queue(move |world: &mut World| paste(world, &scene, line, delta));
                        });
                    },
                    (clipboard.clone(), line, delta),
                )
                .unwrap();
        }
        let ids: Vec<_> = world.query::<&NoteId>().iter(world).copied().collect();
        assert_eq!(ids.len(), 4);
        assert!(ids.iter().all(|id| !old_ids.contains(id)));
        let tracks: Vec<_> = world
            .query::<(Entity, &CurveNoteTrackFrom, &CurveNoteTrackTo)>()
            .iter(world)
            .map(|(entity, from, to)| (entity, from.0, to.0))
            .collect();
        assert_eq!(tracks.len(), 2);
        for (track, from, to) in tracks {
            assert!(world
                .get::<CurveNoteTracksFrom>(from)
                .unwrap()
                .iter()
                .any(|entity| entity == track));
            assert!(world
                .get::<CurveNoteTracksTo>(to)
                .unwrap()
                .iter()
                .any(|entity| entity == track));
            assert_eq!(world.get::<ChildOf>(from).unwrap().parent(), line);
            assert_eq!(world.get::<ChildOf>(track).unwrap().parent(), line);
            assert_eq!(
                world.get::<Note>(to).unwrap().beat - world.get::<Note>(from).unwrap().beat,
                Beat::from(3.0)
            );
        }
        world
            .resource_scope(|world, mut history: Mut<EditorHistory>| history.undo(world))
            .unwrap();
        assert_eq!(world.query::<&Note>().iter(world).count(), 2);
        world
            .resource_scope(|world, mut history: Mut<EditorHistory>| history.redo(world))
            .unwrap();
        assert_eq!(world.query::<&Note>().iter(world).count(), 4);
        assert!(ids
            .iter()
            .all(|id| world.resource::<IdIndex>().entity(id.uuid()).is_some()));

        let track = world
            .query::<(Entity, &CurveNoteTrackTo)>()
            .iter(world)
            .next()
            .unwrap()
            .0;
        world.entity_mut(track).insert(Selected);
        copy_system(world).unwrap();
        let counts = &world.resource::<EditorClipboard>().counts;
        assert_eq!((counts.notes, counts.events, counts.tracks), (2, 0, 1));
    }
}
