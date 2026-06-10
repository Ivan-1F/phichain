use crate::audio::{load_audio, open_and_decode_audio, LoadAudioError};
use crate::illustration::{load_illustration, open_illustration};
use crate::loader::load_line;
use bevy::app::App;
use bevy::prelude::{Commands, Component, Entity, Event, Plugin, Query, Update};
use bevy::tasks::futures_lite::future;
use bevy::tasks::{block_on, AsyncComputeTaskPool, IoTaskPool, Task};
use bevy_kira_audio::prelude::StaticSoundData;
use image::{DynamicImage, ImageResult};
use phichain_chart::project::Project;
use phichain_chart::serialization::{ParseChartError, PhichainChart};
use std::time::{Duration, Instant};
use thiserror::Error;

pub struct ProjectData {
    pub duration: Duration,
    pub project: Project,
    pub chart: PhichainChart,
    illustration: Option<ImageResult<DynamicImage>>,
    sound: StaticSoundData,
}

/// Payload of a successful [`ProjectLoadingResult`]
pub struct LoadedProject {
    pub duration: Duration,
    pub project: Project,
}

#[derive(Error, Debug)]
pub enum LoadProjectError {
    #[error("cannot open chart")]
    CannotOpenChart(#[from] std::io::Error),
    #[error("invalid chart")]
    InvalidChart(#[from] serde_json::Error),
    #[error("migration failed")]
    MigrationFailed(#[from] anyhow::Error),
    #[error("cannot load audio")]
    CannotLoadAudio(#[from] LoadAudioError),
}

impl From<ParseChartError> for LoadProjectError {
    fn from(error: ParseChartError) -> Self {
        match error {
            ParseChartError::InvalidChart(error) => Self::InvalidChart(error),
            ParseChartError::MigrationFailed(error) => Self::MigrationFailed(error),
        }
    }
}

type Result<T> = std::result::Result<T, LoadProjectError>;

type LoadingProjectTask = Task<Result<ProjectData>>;

#[derive(Component)]
pub struct LoadingProject(LoadingProjectTask);

#[derive(Event)]
pub struct ProjectLoadingResult(pub Result<LoadedProject>);

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

        // decode audio and illustration in parallel with chart parsing
        let compute_pool = AsyncComputeTaskPool::get();
        let illustration_task = project
            .path
            .illustration_path()
            .map(|path| compute_pool.spawn(async move { open_illustration(path) }));
        // music_path has been checked in Project::load()
        let audio_path = project.path.music_path().unwrap();
        let audio_task = compute_pool.spawn(async move { open_and_decode_audio(audio_path) });

        let json = std::fs::read_to_string(project.path.chart_path())?;
        let chart = PhichainChart::from_json_str(&json)?;
        drop(json);

        let illustration = match illustration_task {
            Some(task) => Some(task.await),
            None => None,
        };
        let sound = audio_task.await?;

        Ok(ProjectData {
            duration: start.elapsed(),
            project,
            chart,
            illustration,
            sound,
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
                    let ProjectData {
                        duration,
                        project,
                        chart,
                        illustration,
                        sound,
                    } = data;

                    load_audio(sound, &mut commands);

                    // TODO: handle Some(Err)
                    if let Some(Ok(illustration)) = illustration {
                        load_illustration(illustration, &mut commands);
                    }

                    let PhichainChart {
                        offset,
                        bpm_list,
                        lines,
                        ..
                    } = chart;

                    commands.insert_resource(offset);
                    commands.insert_resource(bpm_list);

                    for line in lines {
                        load_line(line, &mut commands, None);
                    }

                    commands.trigger(ProjectLoadingResult(Ok(LoadedProject {
                        duration,
                        project,
                    })));
                }
                Err(error) => {
                    commands.trigger(ProjectLoadingResult(Err(error)));
                }
            }
        }
    }
}
