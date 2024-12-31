use crate::events::curve_note_track::{DespawnCurveNoteTrackEvent, SpawnCurveNoteTrackEvent};
use crate::events::EditorEvent;
use bevy::hierarchy::Parent;
use bevy::prelude::{debug, Entity, World};
use phichain_game::curve_note_track::CurveNoteTrack;
use undo::Edit;

#[derive(Debug, Clone)]
pub struct CreateCurveNoteTrack {
    pub line_entity: Entity,
    pub track: CurveNoteTrack,

    pub track_entity: Option<Entity>,
}

impl CreateCurveNoteTrack {
    pub fn new(line: Entity, track: CurveNoteTrack) -> Self {
        Self {
            line_entity: line,
            track,
            track_entity: None,
        }
    }
}

impl Edit for CreateCurveNoteTrack {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        let entity = SpawnCurveNoteTrackEvent::builder()
            .track(self.track.clone())
            .line_entity(self.line_entity)
            .maybe_target(self.track_entity)
            .build()
            .run(target);

        self.track_entity = Some(entity);
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some(entity) = self.track_entity {
            if target.get_entity(entity).is_none() {
                debug!(
                    "skipping undo `CreateCurveNoteTrack`, the track has been removed internally"
                );
                self.track_entity.take();
                return;
            }

            DespawnCurveNoteTrackEvent::builder()
                .target(entity)
                .keep_entity(true)
                .build()
                .run(target);
        }
    }
}

#[derive(Debug, Clone)]
pub struct RemoveCurveNoteTrack {
    pub entity: Entity,
    pub track: Option<(CurveNoteTrack, Entity)>,
}

impl RemoveCurveNoteTrack {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            track: None,
        }
    }
}

impl Edit for RemoveCurveNoteTrack {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        let track = target.entity(self.entity).get::<CurveNoteTrack>().cloned();
        let parent = target.entity(self.entity).get::<Parent>().map(|x| x.get());
        self.track = Some((track.unwrap(), parent.unwrap()));
        DespawnCurveNoteTrackEvent::builder()
            .target(self.entity)
            .keep_entity(true)
            .build()
            .run(target);
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some((track, line_entity)) = self.track.clone() {
            SpawnCurveNoteTrackEvent::builder()
                .target(self.entity)
                .track(track)
                .line_entity(line_entity)
                .build()
                .run(target);
        }
    }
}
