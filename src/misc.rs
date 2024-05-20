use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{bail, Context};
use bevy::prelude::*;

pub struct MiscPlugin;

impl Plugin for MiscPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorkingDirectory(
            std::env::current_dir().expect("Failed to locate working directory"),
        ));
    }
}

/// Representing the current working directory (parent directory of phichain executable)
#[derive(Resource, Debug)]
pub struct WorkingDirectory(pub PathBuf);

impl WorkingDirectory {
    fn directory(&self, path: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
        let new_path = self.0.join(&path);

        if !new_path.exists() {
            fs::create_dir(&new_path).context(format!("Failed to create directory: {:?}", path.as_ref()))?;
        }
        if !new_path.is_dir() {
            bail!("Expected a directory at {:?}, found a file", path.as_ref());
        }

        Ok(new_path)
    }

    pub fn screenshot(&self) -> anyhow::Result<PathBuf> {
        self.directory("screenshots")
    }
}
