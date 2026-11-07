use crate::editing::history::Edits;
use crate::notification::{ToastsExt, ToastsStorage};
use crate::schedule::EditorSet;
use crate::timeline::click::TimelineClick;
use crate::timeline::pending::{spawn_pending, TimelinePending};
use crate::timeline::pending_status::{PendingHint, PendingStatus};
use bevy::prelude::*;
use phichain_chart::curve_note_track::CurveNoteTrackOptions;
use phichain_chart::note::Note;
use phichain_game::curve_note_track::{CurveNote, CurveNoteTrackFrom, CurveNoteTrackTo};
use phichain_game::Pending;

pub struct CreateCurveNoteTrackPlugin;

impl Plugin for CreateCurveNoteTrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, complete_curve_note_track.in_set(EditorSet::Edit));
    }
}

/// Start a curve note track from the note `from`, completed by clicking the end note on `timeline`
pub fn start_curve_note_track(
    commands: &mut Commands,
    timeline: Entity,
    from: Entity,
    line: Entity,
) {
    spawn_pending(
        commands,
        timeline,
        PendingStatus {
            title: t!("tab.timeline.pending.curve_note_track").into(),
            hints: vec![PendingHint::text(t!("tab.timeline.pending.pick_end"))],
        },
        (CurveNoteTrackFrom(from), ChildOf(line)),
    );
}

fn complete_curve_note_track(
    mut commands: Commands,
    mut clicks: MessageReader<TimelineClick>,
    mut edits: Edits,
    mut toasts: ResMut<ToastsStorage>,
    timeline_pending: Query<&TimelinePending>,
    tracks: Query<(&CurveNoteTrackFrom, &CurveNoteTrackOptions)>,
    notes: Query<&ChildOf, (With<Note>, Without<CurveNote>, Without<Pending>)>,
) {
    for click in clicks.read() {
        let Some((preview, (from, options))) = timeline_pending
            .get(click.timeline)
            .ok()
            .and_then(TimelinePending::entity)
            .and_then(|preview| Some((preview, tracks.get(preview).ok()?)))
        else {
            continue;
        };
        let (from, to) = (from.0, click.entity);
        if !notes.contains(to) {
            continue;
        }
        if from == to {
            toasts.info(t!("tab.timeline.curve_note_track.same_endpoint"));
            continue;
        }
        let Ok(line) = notes.get(from).map(ChildOf::parent) else {
            continue;
        };
        let options = options.clone();
        commands.entity(preview).despawn();
        edits.once(t!("history.create_track"), move |commands| {
            commands.spawn((
                options,
                CurveNoteTrackFrom(from),
                CurveNoteTrackTo(to),
                ChildOf(line),
            ));
        });
        break;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editing::history::{open_document, EditorHistory, HistoryPlugin};
    use crate::id_index::IdIndexPlugin;
    use crate::selection::SelectedLine;
    use crate::timeline::pending::PendingOf;
    use crate::timeline::target::FollowSelectedLine;
    use bevy::ecs::system::RunSystemOnce;
    use phichain_chart::beat::Beat;
    use phichain_chart::line::Line;
    use phichain_chart::note::NoteKind;

    struct Fixture {
        app: App,
        line: Entity,
        owner: Entity,
        from: Entity,
        to: Entity,
    }

    fn fixture() -> Fixture {
        let mut app = App::new();
        app.add_plugins((IdIndexPlugin, HistoryPlugin, CreateCurveNoteTrackPlugin))
            .init_resource::<ToastsStorage>()
            .add_message::<TimelineClick>();
        let world = app.world_mut();
        let line = world.spawn(Line::default()).id();
        let note = |beat: f32| Note::new(NoteKind::Tap, true, Beat::from(beat), 0.0, 1.0);
        let from = world.spawn((note(0.0), ChildOf(line))).id();
        let to = world.spawn((note(1.0), ChildOf(line))).id();
        let owner = world.spawn(FollowSelectedLine).id();
        world.insert_resource(SelectedLine(line));
        open_document(world);
        world
            .run_system_once(move |mut commands: Commands| {
                start_curve_note_track(&mut commands, owner, from, line);
            })
            .unwrap();
        Fixture {
            app,
            line,
            owner,
            from,
            to,
        }
    }

    fn click(app: &mut App, timeline: Entity, entity: Entity) {
        app.world_mut()
            .write_message(TimelineClick { timeline, entity });
        app.update();
    }

    fn tracks(world: &mut World) -> Vec<(Entity, Entity, Entity)> {
        world
            .query_filtered::<(&CurveNoteTrackFrom, &CurveNoteTrackTo, &ChildOf), Without<Pending>>(
            )
            .iter(world)
            .map(|(from, to, line)| (from.0, to.0, line.parent()))
            .collect()
    }

    fn pending_tracks(world: &mut World) -> Vec<(Entity, Entity, Entity)> {
        world
            .query::<(&CurveNoteTrackFrom, &PendingOf, &ChildOf)>()
            .iter(world)
            .map(|(from, owner, line)| (from.0, owner.0, line.parent()))
            .collect()
    }

    #[test]
    fn starting_creates_a_pending_track_on_the_timeline() {
        let mut fixture = fixture();
        let world = fixture.app.world_mut();
        assert_eq!(
            pending_tracks(world),
            vec![(fixture.from, fixture.owner, fixture.line)]
        );
    }

    #[test]
    fn clicking_a_note_on_the_owner_timeline_completes_the_track() {
        let mut fixture = fixture();
        click(&mut fixture.app, fixture.owner, fixture.to);

        let world = fixture.app.world_mut();
        assert!(pending_tracks(world).is_empty());
        assert_eq!(
            tracks(world),
            vec![(fixture.from, fixture.to, fixture.line)]
        );
        assert_eq!(world.resource::<EditorHistory>().entries().count(), 1);
    }

    #[test]
    fn clicks_on_other_timelines_or_on_the_start_note_keep_the_track_pending() {
        let mut fixture = fixture();
        let other = fixture.app.world_mut().spawn_empty().id();
        click(&mut fixture.app, other, fixture.to);
        click(&mut fixture.app, fixture.owner, fixture.from);

        let world = fixture.app.world_mut();
        assert_eq!(pending_tracks(world).len(), 1);
        assert!(tracks(world).is_empty());
    }
}
