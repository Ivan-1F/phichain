use anyhow::{bail, Context};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use std::{fs::File, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct ProjectMeta {
    pub composer: String,
    pub charter: String,
    pub illustrator: String,
    pub name: String,
    pub level: String,
}

#[derive(Resource)]
pub struct Project {
    pub root_dir: PathBuf,
    pub meta: ProjectMeta,
}

impl Project {
    pub fn load(root_dir: PathBuf) -> anyhow::Result<Self> {
        ProjectPath(root_dir).into_project()
    }
}

struct ProjectPath(PathBuf);

impl ProjectPath {
    pub fn chart_path(&self) -> PathBuf {
        self.0.join("chart.json")
    }

    pub fn music_path(&self) -> PathBuf {
        self.0.join("music.wav")
    }

    pub fn meta_path(&self) -> PathBuf {
        self.0.join("meta.json")
    }

    pub fn into_project(self) -> anyhow::Result<Project> {
        if !self.chart_path().is_file() {
            bail!("chart.json is missing");
        }
        if !self.music_path().is_file() {
            bail!("music.wav is missing");
        }
        if !self.meta_path().is_file() {
            bail!("meta.json is missing");
        }

        let meta_file = File::open(self.meta_path()).context("Failed to open meta file")?;
        let meta: ProjectMeta = serde_json::from_reader(meta_file).context("Invalid meta file")?;

        Ok(Project {
            root_dir: self.0,
            meta,
        })
    }
}

/// A [Condition] represents the project is loaded
pub fn project_loaded() -> impl Condition<()> {
    resource_exists::<Project>.and_then(|| true)
}

/// A [Condition] represents the project is not loaded
pub fn project_not_loaded() -> impl Condition<()> {
    resource_exists::<Project>.map(|x| !x)
}