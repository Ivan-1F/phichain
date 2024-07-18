use bevy::prelude::*;
use phichain_chart::bpm_list::BpmList;
use phichain_chart::note::Note;

use crate::{project::project_loaded, timing::ChartTime};

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameScore::default())
            .add_systems(Update, update_score_system.run_if(project_loaded()));
    }
}

#[derive(Resource, Debug, Default)]
pub struct GameScore {
    combo: u32,
    note_amount: u32,
}

impl GameScore {
    pub fn combo(&self) -> u32 {
        self.combo
    }

    pub fn score(&self) -> f32 {
        match self.note_amount {
            0 => 0.0,
            _ => (100_0000.0 * self.combo as f32 / self.note_amount as f32).round(),
        }
    }

    pub fn score_text(&self) -> String {
        format!("{:07}", self.score())
    }
}

fn update_score_system(
    mut score: ResMut<GameScore>,
    note_query: Query<&Note>,
    time: Res<ChartTime>,
    bpm_list: Res<BpmList>,
) {
    let notes: Vec<_> = note_query.iter().collect();
    score.combo = notes
        .iter()
        .filter(|note| bpm_list.time_at(note.beat) <= time.0)
        .collect::<Vec<_>>()
        .len() as u32;

    score.note_amount = notes.len() as u32;
}
