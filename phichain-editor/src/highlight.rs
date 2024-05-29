use bevy::prelude::*;
use bevy::utils::HashMap;
use phichain_chart::beat::Beat;
use phichain_chart::note::Note;

pub struct HighlightPlugin;

impl Plugin for HighlightPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HighlightedBeat>().add_systems(
            PostUpdate,
            (calc_highlighted_beat_system, mark_highlight_system).chain(),
        );
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Highlighted;

#[derive(Resource, Default, Debug, Clone)]
pub struct HighlightedBeat(HashMap<Beat, u32>);

fn calc_highlighted_beat_system(
    query: Query<&Note>,
    mut highlighted_beat: ResMut<HighlightedBeat>,
) {
    highlighted_beat.0.clear();
    for note in &query {
        let counter = highlighted_beat.0.entry(note.beat.reduced()).or_insert(0);
        *counter += 1;
    }
}

fn mark_highlight_system(
    mut commands: Commands,
    query: Query<(Entity, &Note)>,
    highlighted_beat: ResMut<HighlightedBeat>,
) {
    for (entity, note) in &query {
        if highlighted_beat.0.contains_key(&note.beat.reduced())
            && highlighted_beat.0[&note.beat.reduced()] > 1
        {
            commands.entity(entity).insert(Highlighted);
        } else {
            commands.entity(entity).remove::<Highlighted>();
        }
    }
}
