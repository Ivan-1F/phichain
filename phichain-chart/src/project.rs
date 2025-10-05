use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProjectMeta {
    pub composer: String,
    pub charter: String,
    pub illustrator: String,
    pub name: String,
    pub level: String,
}

#[derive(Error, Debug)]
pub enum LoadProjectError {
    #[error("missing {0} file")]
    MissingFile(&'static str),
    #[error("cannot open meta.json")]
    CannotOpenMeta(#[from] std::io::Error),
    #[error("invalid meta.json")]
    InvalidMeta(#[from] serde_json::Error),
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
    pub fn load(root_dir: PathBuf) -> Result<Self, LoadProjectError> {
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

    pub fn into_project(self) -> Result<Project, LoadProjectError> {
        if !self.chart_path().is_file() {
            return Err(LoadProjectError::MissingFile("chart.json"));
        }
        if !self
            .music_path()
            .ok_or(LoadProjectError::MissingFile("music.[wav|mp3|ogg|flac]"))?
            .is_file()
        {
            return Err(LoadProjectError::MissingFile("music.[wav|mp3|ogg|flac]"));
        }
        if !self.meta_path().is_file() {
            return Err(LoadProjectError::MissingFile("meta.json"));
        }

        let meta_file = File::open(self.meta_path())?;
        let meta: ProjectMeta = serde_json::from_reader(meta_file)?;

        Ok(Project {
            path: self,
            meta,
            id: Uuid::new_v4(),
        })
    }
}
