use crate::events::{EditorEvent, EditorEventAppExt};
use crate::utils::entity::replace_with_empty;
use bevy::app::{App, Plugin};
use bevy::log::debug;
use bevy::prelude::{ChildOf, Entity, Message, World};
use bon::Builder;
use phichain_chart::curve_note_track::CurveNoteTrackOptions;
use phichain_chart::id::CurveNoteTrackId;
use phichain_game::curve_note_track::{CurveNote, CurveNoteTrackFrom, CurveNoteTrackTo};

pub struct CurveNoteTrackEventPlugin;

impl Plugin for CurveNoteTrackEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_editor_event::<SpawnCurveNoteTrackEvent>()
            .add_editor_event::<DespawnCurveNoteTrackEvent>();
    }
}

#[derive(Debug, Clone, Message, Builder)]
pub struct SpawnCurveNoteTrackEvent {
    options: CurveNoteTrackOptions,
    from: Entity,
    to: Entity,
    id: CurveNoteTrackId,
    line_entity: Entity,
    target: Option<Entity>,
}

impl EditorEvent for SpawnCurveNoteTrackEvent {
    type Output = Entity;

    fn run(self, world: &mut World) -> Self::Output {
        debug!("spawn curve note track on line {:?}", self.line_entity);
        let id = match self.target {
            None => world.spawn_empty().id(),
            Some(target) => target,
        };
        world
            .entity_mut(id)
            .insert(self.options)
            .insert(self.id)
            .insert(CurveNoteTrackFrom(self.from))
            .insert(CurveNoteTrackTo(self.to))
            .insert(ChildOf(self.line_entity))
            .id()
    }
}

#[derive(Debug, Clone, Message, Builder)]
pub struct DespawnCurveNoteTrackEvent {
    target: Entity,
    #[builder(default = false)]
    keep_entity: bool,
}

impl EditorEvent for DespawnCurveNoteTrackEvent {
    type Output = ();

    fn run(self, world: &mut World) -> Self::Output {
        // generated notes belong to the track and must not outlive it;
        // tombstoning would otherwise unlink their CurveNote marker and
        // leak them into serialization as ordinary notes
        let generated: Vec<Entity> = world
            .query::<(Entity, &CurveNote)>()
            .iter(world)
            .filter(|(_, note)| note.0 == self.target)
            .map(|(entity, _)| entity)
            .collect();
        for entity in generated {
            world.entity_mut(entity).despawn();
        }

        debug!(
            "despawned CNT {:?}{}",
            self.target,
            if self.keep_entity {
                " (keep entity)"
            } else {
                ""
            }
        );
        if self.keep_entity {
            replace_with_empty(world, self.target);
        } else {
            world.entity_mut(self.target).despawn();
        }
    }
}
