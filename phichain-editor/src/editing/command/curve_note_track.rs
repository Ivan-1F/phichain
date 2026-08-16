use crate::events::curve_note_track::{DespawnCurveNoteTrackEvent, SpawnCurveNoteTrackEvent};
use crate::events::EditorEvent;
use bevy::prelude::{debug, ChildOf, Entity, World};
use phichain_chart::curve_note_track::CurveNoteTrackOptions;
use phichain_chart::id::CurveNoteTrackId;
use phichain_game::curve_note_track::{CurveNoteTrackFrom, CurveNoteTrackTo};
use undo::Edit;

#[derive(Debug, Clone)]
pub struct CreateCurveNoteTrack {
    pub line_entity: Entity,
    pub options: CurveNoteTrackOptions,
    pub from: Entity,
    pub to: Entity,
    pub track_id: CurveNoteTrackId,

    pub track_entity: Option<Entity>,
}

impl CreateCurveNoteTrack {
    pub fn new(line: Entity, from: Entity, to: Entity, options: CurveNoteTrackOptions) -> Self {
        Self {
            line_entity: line,
            options,
            from,
            to,
            track_id: CurveNoteTrackId::new(),

            track_entity: None,
        }
    }
}

impl Edit for CreateCurveNoteTrack {
    type Target = World;
    type Output = ();

    fn edit(&mut self, target: &mut Self::Target) -> Self::Output {
        let entity = SpawnCurveNoteTrackEvent::builder()
            .options(self.options.clone())
            .from(self.from)
            .to(self.to)
            .id(self.track_id)
            .line_entity(self.line_entity)
            .maybe_target(self.track_entity)
            .build()
            .run(target);

        self.track_entity = Some(entity);
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some(entity) = self.track_entity {
            if target.get_entity(entity).is_ok() {
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
    #[allow(clippy::type_complexity)]
    pub track: Option<(
        CurveNoteTrackOptions,
        CurveNoteTrackFrom,
        CurveNoteTrackTo,
        CurveNoteTrackId,
        Entity,
    )>,
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
        let entity = target.entity(self.entity);
        let options = entity.get::<CurveNoteTrackOptions>().cloned();
        let from = entity.get::<CurveNoteTrackFrom>().copied();
        let to = entity.get::<CurveNoteTrackTo>().copied();
        let track_id = entity.get::<CurveNoteTrackId>().copied();
        let parent = entity.get::<ChildOf>().map(|x| x.parent());
        self.track = Some((
            options.unwrap(),
            from.unwrap(),
            to.unwrap(),
            track_id.unwrap(),
            parent.unwrap(),
        ));
        DespawnCurveNoteTrackEvent::builder()
            .target(self.entity)
            .keep_entity(true)
            .build()
            .run(target);
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        if let Some((options, from, to, track_id, line_entity)) = self.track.clone() {
            SpawnCurveNoteTrackEvent::builder()
                .target(self.entity)
                .options(options)
                .from(from.0)
                .to(to.0)
                .id(track_id)
                .line_entity(line_entity)
                .build()
                .run(target);
        }
    }
}
