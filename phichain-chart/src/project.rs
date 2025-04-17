use anyhow::{anyhow, bail, Context};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProjectMeta {
    pub composer: String,
    pub charter: String,
    pub illustrator: String,
    pub name: String,
    pub level: String,
}

#[cfg_attr(feature = "bevy", derive(bevy::prelude::Resource))]
#[derive(Debug, Clone)]
pub struct Project {
    pub path: ProjectPath,
    pub meta: ProjectMeta,

    /// A unique uuid represents a project session.
    pub id: Uuid,
}

impl Project {
    pub fn load(root_dir: PathBuf) -> anyhow::Result<Self> {
        ProjectPath(root_dir).into_project()
    }
}

#[derive(Debug, Clone)]
pub struct ProjectPath(pub PathBuf);

impl ProjectPath {
    pub fn chart_path(&self) -> PathBuf {
        self.0.join("chart.json")
    }

    pub fn sub_path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.0.join(path)
    }

    fn find_file(&self, name: &str, allowed_extensions: &[impl ToString]) -> Option<PathBuf> {
        std::fs::read_dir(&self.0)
            .ok()?
            .filter_map(Result::ok)
            .map(|x| x.path())
            .find(|path| {
                path.is_file()
                    && path.file_stem() == Some(name.as_ref())
                    && path.extension().is_some_and(|ext| {
                        allowed_extensions
                            .iter()
                            .any(|allowed| *allowed.to_string() == *ext)
                    })
            })
    }

    pub fn music_path(&self) -> Option<PathBuf> {
        self.find_file("music", &["wav", "mp3", "ogg", "flac"])
    }

    pub fn illustration_path(&self) -> Option<PathBuf> {
        self.find_file("illustration", &["png", "jpg", "jpeg"])
    }

    pub fn meta_path(&self) -> PathBuf {
        self.0.join("meta.json")
    }

    pub fn into_project(self) -> anyhow::Result<Project> {
        if !self.chart_path().is_file() {
            bail!("chart.json is missing");
        }
        if !self
            .music_path()
            .ok_or(anyhow!("Could not find music file in project"))?
            .is_file()
        {
            bail!("music.wav is missing");
        }
        if !self.meta_path().is_file() {
            bail!("meta.json is missing");
        }

        let meta_file = File::open(self.meta_path()).context("Failed to open meta file")?;
        let meta: ProjectMeta = serde_json::from_reader(meta_file).context("Invalid meta file")?;

        Ok(Project {
            path: self,
            meta,
            id: Uuid::new_v4(),
        })
    }
}
