use crate::{GameConfig, GameSet};
use bevy::prelude::*;
use bevy::utils::HashMap;
use phichain_chart::beat::Beat;
use phichain_chart::note::Note;

pub struct HighlightPlugin;

impl Plugin for HighlightPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HighlightedBeat>().add_systems(
            Update,
            (calc_highlighted_beat_system, mark_highlight_system)
                .chain()
                .in_set(GameSet),
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

    settings: Res<GameConfig>,
) {
    for (entity, note) in &query {
        #[allow(clippy::collapsible_else_if)] // keeping the current form for better readability
        if highlighted_beat.0.contains_key(&note.beat.reduced())
            && highlighted_beat.0[&note.beat.reduced()] > 1
            && settings.multi_highlight
        {
            if let Some(mut entity) = commands.get_entity(entity) {
                entity.try_insert(Highlighted);
            }
        } else {
            if let Some(mut entity) = commands.get_entity(entity) {
                entity.remove::<Highlighted>();
            }
        }
    }
}
