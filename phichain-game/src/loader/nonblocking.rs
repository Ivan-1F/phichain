use crate::loader::load_line;
use anyhow::{Context, Result};
use bevy::app::App;
use bevy::prelude::{Commands, Component, Entity, Event, Plugin, Query, Update};
use bevy::tasks::futures_lite::future;
use bevy::tasks::{block_on, IoTaskPool, Task};
use phichain_chart::migration::migrate;
use phichain_chart::project::Project;
use phichain_chart::serialization::PhichainChart;
use serde_json::Value;
use std::fs::File;
use std::time::{Duration, Instant};

pub struct ProjectData {
    duration: Duration,
    project: Project,
    chart: PhichainChart,
}

pub type LoadingProjectTask = Task<Result<ProjectData>>;

#[derive(Component)]
pub struct LoadingProject(LoadingProjectTask);

#[derive(Event)]
pub struct ProjectLoaded(ProjectData);

impl ProjectLoaded {
    pub fn duration(&self) -> Duration {
        self.0.duration
    }

    pub fn project(&self) -> &Project {
        &self.0.project
    }

    pub fn chart(&self) -> &PhichainChart {
        &self.0.chart
    }
}

pub struct NonblockingLoaderPlugin;

impl Plugin for NonblockingLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_tasks_system);
    }
}

pub fn load_project(project: &Project, commands: &mut Commands) {
    let project = project.clone();

    let thread_pool = IoTaskPool::get();

    let task: LoadingProjectTask = thread_pool.spawn(async move {
        let start = Instant::now();

        let file = File::open(project.path.chart_path())?;
        let chart: Value = serde_json::from_reader(file).context("Failed to load chart")?;
        let migrated = migrate(&chart).context("Migration failed")?;
        let chart: PhichainChart =
            serde_json::from_value(migrated).context("Failed to deserialize chart")?;

        Ok(ProjectData {
            duration: start.elapsed(),
            project: project.clone(),
            chart,
        })
    });

    commands.spawn(LoadingProject(task));
}

pub fn handle_tasks_system(
    mut commands: Commands,
    mut loading_tasks: Query<(Entity, &mut LoadingProject)>,
) {
    for (entity, mut task) in &mut loading_tasks {
        if let Some(result) = block_on(future::poll_once(&mut task.0)) {
            // despawn the task
            commands.entity(entity).despawn();

            // load the chart
            match result {
                Ok(data) => {
                    commands.insert_resource(data.chart.offset);
                    commands.insert_resource(data.chart.bpm_list.clone());

                    let mut first_line_id: Option<Entity> = None;
                    for line in &data.chart.lines {
                        let id = load_line(line.clone(), &mut commands, None);
                        if first_line_id.is_none() {
                            first_line_id = Some(id)
                        }
                    }

                    commands.trigger(ProjectLoaded(data));
                }
                Err(_) => {
                    // TODO: handle error
                }
            }
        }
    }
}
