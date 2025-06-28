use crate::misc::WorkingDirectory;
use bevy::prelude::*;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub struct TranslationPlugin;

impl Plugin for TranslationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_translation_system);
    }
}

#[derive(Resource, Debug, Serialize, Deserialize)]
pub struct Languages(pub IndexMap<String, String>);

fn load_translation_system(mut commands: Commands, working_directory: Res<WorkingDirectory>) {
    let meta = working_directory.0.join("lang/meta.json");
    let languages: Languages =
        serde_json::from_reader(std::fs::File::open(meta).expect("Failed to open language meta"))
            .expect("Failed to parse language meta");

    commands.insert_resource(languages);
}
