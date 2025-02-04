use crate::events::{EditorEvent, EditorEventAppExt};
use crate::utils::entity::replace_with_empty;
use bevy::app::{App, Plugin};
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::log::debug;
use bevy::prelude::{BuildChildren, Entity, Event, World};
use bon::Builder;
use phichain_game::curve_note_track::CurveNoteTrack;

pub struct CurveNoteTrackEventPlugin;

impl Plugin for CurveNoteTrackEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_editor_event::<SpawnCurveNoteTrackEvent>()
            .add_editor_event::<DespawnCurveNoteTrackEvent>();
    }
}

#[derive(Debug, Clone, Event, Builder)]
pub struct SpawnCurveNoteTrackEvent {
    track: CurveNoteTrack,
    line_entity: Entity,
    target: Option<Entity>,
}

impl EditorEvent for SpawnCurveNoteTrackEvent {
    type Output = Entity;

    fn run(self, world: &mut World) -> Self::Output {
        debug!("spawn curve note track");
        match self.target {
            None => {
                debug!("spawned CNT {:?} on new entity", self.track);
            }
            Some(target) => {
                debug!("spawned CNT {:?} on entity {:?}", self.track, target);
            }
        }
        let id = match self.target {
            None => world.spawn_empty().id(),
            Some(target) => target,
        };
        world
            .entity_mut(id)
            .insert(self.track)
            .set_parent(self.line_entity)
            .id()
    }
}

#[derive(Debug, Clone, Event, Builder)]
pub struct DespawnCurveNoteTrackEvent {
    target: Entity,
    #[builder(default = false)]
    keep_entity: bool,
}

impl EditorEvent for DespawnCurveNoteTrackEvent {
    type Output = ();

    fn run(self, world: &mut World) -> Self::Output {
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
            world.entity_mut(self.target).despawn_recursive();
        }
    }
}
