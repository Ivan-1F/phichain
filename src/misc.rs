use bevy::prelude::*;

pub struct MiscPlugin;

impl Plugin for MiscPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorkingDirectory(std::env::current_dir().expect("Failed to locate working directory")));
    }
}

/// Representing the current working directory (parent directory of phichain executable)
#[derive(Resource, Debug)]
pub struct WorkingDirectory(pub std::path::PathBuf);
