use crate::curve_note_track::CurveNoteTrack as GameCurveNoteTrack;
use crate::event::Events;
use bevy::app::App;
use bevy::ecs::component::HookContext;
use bevy::ecs::spawn::{SpawnIter, SpawnWith};
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::{Bundle, ChildSpawner, Children, Component, Plugin, Resource, SpawnRelated};
use phichain_chart::line::Line;
use phichain_chart::serialization::SerializedLine;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Resource, Default)]
struct OrderGen(AtomicU64);
impl OrderGen {
    #[inline]
    fn next(&self) -> u64 {
        self.0.fetch_add(1, Ordering::Relaxed)
    }
}

/// This is a temporary workaround to maintain line order
#[derive(Component, Default, Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
#[component(on_add = LineOrder::on_add)]
pub struct LineOrder(pub u64);

impl LineOrder {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let next = world.resource::<OrderGen>().next();
        if let Some(mut order) = world.entity_mut(ctx.entity).get_mut::<LineOrder>() {
            *order = LineOrder(next);
        }
    }
}

pub struct LinePlugin;

impl Plugin for LinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OrderGen>()
            .register_required_components::<Line, LineOrder>();
    }
}

/// Returns a Bundle that spawns a line entity together with its notes,
/// curve-note tracks (CNT), events, and recursively its child lines.
///
/// Usage
/// - `commands.spawn(line_bundle(serialized_line));`
/// - `world.spawn(line_bundle(serialized_line));`
/// - `parent.spawn(line_bundle(serialized_line));`
/// - ...
pub fn line_bundle(line: SerializedLine) -> impl Bundle {
    let SerializedLine {
        line: line_comp,
        notes,
        events,
        children,
        curve_note_tracks,
    } = line;

    (
        line_comp,
        Children::spawn((
            SpawnWith(move |parent: &mut ChildSpawner| {
                let mut note_entities = Vec::with_capacity(notes.len());
                for note in notes {
                    let id = parent.spawn(note).id();
                    note_entities.push(id);
                }

                for track in curve_note_tracks {
                    if let (Some(&from), Some(&to)) =
                        (note_entities.get(track.from), note_entities.get(track.to))
                    {
                        parent.spawn(GameCurveNoteTrack {
                            from: Some(from),
                            to: Some(to),
                            options: track.options,
                        });
                    }
                }
            }),
            SpawnWith(move |parent: &mut ChildSpawner| {
                for child in children {
                    parent.spawn(line_bundle(child));
                }
            }),
        )),
        Events::spawn(SpawnIter(events.into_iter())),
    )
}
