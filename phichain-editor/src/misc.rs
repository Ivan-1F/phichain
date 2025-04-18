use bevy::prelude::*;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub struct MiscPlugin;

impl MiscPlugin {
    // https://github.com/bevyengine/bevy/blob/dc3b4b6c850898c922dff9fd6d312823e07096f1/crates/bevy_asset/src/io/file_asset_io.rs#L65
    pub fn get_base_path() -> PathBuf {
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            PathBuf::from(manifest_dir)
        } else {
            std::env::current_exe()
                .map(|path| {
                    path.parent()
                        .map(|exe_parent_path| exe_parent_path.to_owned())
                        .unwrap()
                })
                .unwrap()
        }
    }
}

impl Plugin for MiscPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorkingDirectory::default());
    }
}

/// Representing the current working directory (parent directory of phichain executable)
#[derive(Resource, Debug)]
pub struct WorkingDirectory(pub PathBuf);

impl Default for WorkingDirectory {
    fn default() -> Self {
        Self(MiscPlugin::get_base_path())
    }
}

impl WorkingDirectory {
    fn directory(&self, path: impl AsRef<Path>) -> io::Result<PathBuf> {
        let new_path = self.0.join(&path);

        if !new_path.exists() {
            fs::create_dir(&new_path)?;
        }
        if !new_path.is_dir() {
            return Err(io::Error::from(io::ErrorKind::NotADirectory));
        }

        Ok(new_path)
    }

    pub fn screenshot(&self) -> io::Result<PathBuf> {
        self.directory("screenshots")
    }
    pub fn config(&self) -> io::Result<PathBuf> {
        self.directory("config")
    }
    pub fn log(&self) -> io::Result<PathBuf> {
        self.directory("logs")
    }
}
