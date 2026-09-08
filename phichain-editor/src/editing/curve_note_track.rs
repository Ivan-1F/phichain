use crate::notification::{ToastsExt, ToastsStorage};
use crate::selection::Selected;
use crate::GameSet;
use bevy::prelude::*;
use phichain_chart::curve_note_track::CurveNoteTrackOptions;
use phichain_chart::id::CurveNoteTrackId;
use phichain_game::curve_note_track::{CurveNoteTrackFrom, CurveNoteTrackTo};

pub struct CurveNoteTrackPlugin;

impl Plugin for CurveNoteTrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            cancel_pending_curve_note_track_system.in_set(GameSet),
        );

        #[cfg(debug_assertions)]
        app.add_systems(Update, cnt_invariant_system.in_set(GameSet));
    }
}

/// Every CNT entity must be in one of the two legal states:
/// `(From, Options)` or `(From, To, Options, Id)`
#[cfg(debug_assertions)]
fn cnt_invariant_system(world: &mut World) {
    let mut from_query = world.query::<(Entity, &CurveNoteTrackFrom)>();
    for (entity, _) in from_query.iter(world) {
        assert!(
            world.get::<CurveNoteTrackOptions>(entity).is_some(),
            "CNT entity {entity:?} has From but no Options"
        );
    }
    let mut to_query = world.query::<(Entity, &CurveNoteTrackTo)>();
    for (entity, _) in to_query.iter(world) {
        assert!(
            world.get::<CurveNoteTrackFrom>(entity).is_some()
                && world.get::<CurveNoteTrackId>(entity).is_some(),
            "CNT entity {entity:?} has To but lacks From or Id"
        );
    }
}

fn cancel_pending_curve_note_track_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<
        (Entity, Option<&Selected>),
        (With<CurveNoteTrackFrom>, Without<CurveNoteTrackTo>),
    >,
    mut toasts: ResMut<ToastsStorage>,
) {
    for (entity, selected) in &query {
        if keyboard.just_pressed(KeyCode::Escape) || selected.is_none() {
            commands.entity(entity).despawn();
            toasts.info(t!("tab.inspector.curve_note_track.removed.incomplete"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection::{handle_select_event, Select};
    use phichain_game::Pending;

    #[test]
    fn a_curve_preview_remains_selected_until_cancelled() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ToastsStorage>()
            .add_message::<Select>()
            .add_plugins(CurveNoteTrackPlugin)
            .add_systems(Update, handle_select_event.before(GameSet));
        let origin = app.world_mut().spawn_empty().id();
        let preview = app
            .world_mut()
            .spawn((CurveNoteTrackFrom(origin), Pending))
            .id();
        app.world_mut().write_message(Select(vec![preview]));
        app.update();
        assert!(app.world().get::<Selected>(preview).is_some());
        app.update();
        assert!(app.world().get_entity(preview).is_ok());
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        assert!(app.world().get_entity(preview).is_err());
    }
}
