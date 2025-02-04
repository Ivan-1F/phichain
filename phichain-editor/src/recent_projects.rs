use crate::misc::WorkingDirectory;
use crate::project::Project;
use bevy::prelude::{App, Plugin, Resource};
use bevy_persistent::{Persistent, StorageFormat};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentProject {
    pub name: String,
    pub path: PathBuf,
    pub last_opened: DateTime<Local>,
}

impl RecentProject {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            last_opened: Local::now(),
        }
    }
}

impl Ord for RecentProject {
    fn cmp(&self, other: &Self) -> Ordering {
        self.last_opened.cmp(&other.last_opened)
    }
}

impl PartialOrd for RecentProject {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Default, Resource, Serialize, Deserialize)]
pub struct RecentProjects(pub Vec<RecentProject>);

pub trait PersistentRecentProjectsExt {
    /// Push and persist a new [`RecentProject`]. If entry with same path exists, update and persist it's [`last_opened`]
    fn push(&mut self, project: RecentProject);
    /// Remove and persist a [`RecentProject`] at the given index
    fn remove(&mut self, index: usize);
    /// Refresh and persist names of all [`RecentProject`] based on their paths, removing all invalid projects
    fn refresh(&mut self);
}

impl PersistentRecentProjectsExt for Persistent<RecentProjects> {
    fn push(&mut self, project: RecentProject) {
        match self.0.iter_mut().find(|x| x.path == project.path) {
            None => {
                self.0.push(project);
            }
            Some(recent_project) => recent_project.last_opened = Local::now(),
        }
        self.0.sort();
        self.persist().expect("Failed to persist recent projects");
    }

    fn remove(&mut self, index: usize) {
        self.0.remove(index);
        self.persist().expect("Failed to persist recent projects");
    }

    fn refresh(&mut self) {
        self.0.retain_mut(|x| match Project::load(x.path.clone()) {
            Ok(project) => {
                x.name = project.meta.name;
                true
            }
            Err(_) => false,
        });
    }
}

pub struct RecentProjectsPlugin;

impl Plugin for RecentProjectsPlugin {
    fn build(&self, app: &mut App) {
        let config_dir = app
            .world()
            .resource::<WorkingDirectory>()
            .config()
            .expect("Failed to locate config directory");

        let mut resource = Persistent::<RecentProjects>::builder()
            .name("Recent Projects")
            .format(StorageFormat::Yaml)
            .path(config_dir.join("recent_projects.yml"))
            .default(RecentProjects::default())
            .build()
            .expect("Failed to initialize recent projects");
        resource.refresh();

        app.insert_resource(resource);
    }
}
