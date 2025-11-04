use anyhow::Context;
use bevy::prelude::*;

use crate::action::ActionRegistrationExt;
use crate::editing::history::EditorHistory;
use crate::hotkey::modifier::Modifier;
use crate::hotkey::Hotkey;
use crate::notification::{ToastsExt, ToastsStorage};
use crate::recent_projects::{PersistentRecentProjectsExt, RecentProject, RecentProjects};
use crate::spectrogram::{self, Spectrogram};
use crate::telemetry::PushTelemetryEvent;
use bevy::ecs::system::SystemState;
use bevy_kira_audio::{Audio, AudioControl, AudioSource};
use bevy_persistent::Persistent;
use phichain_chart::line::Line;
use phichain_chart::project::OpenProjectError;
pub use phichain_chart::project::{Project, ProjectMeta, ProjectPath};
use phichain_chart::serialization::PhichainChart;
use phichain_game::audio::LoadAudioError;
use phichain_game::loader::nonblocking::{LoadProjectError, ProjectLoadingResult};
use phichain_game::serialization::{serialize_chart, SerializeChartParam, SerializeLineParam};
use serde_json::json;
use std::path::PathBuf;

/// A [Condition] represents the project is loaded
pub fn project_loaded() -> impl Condition<()> {
    resource_exists::<Project>.and(|| true)
}

/// A [Condition] represents the project is not loaded
pub fn project_not_loaded() -> impl Condition<()> {
    IntoSystem::into_system(resource_exists::<Project>.map(|x| !x))
}

pub struct ProjectPlugin;

impl Plugin for ProjectPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadProjectEvent>()
            .add_systems(Update, load_project_system.run_if(project_not_loaded()))
            .add_event::<UnloadProjectEvent>()
            .add_systems(PreUpdate, unload_project_system.run_if(project_loaded()))
            .add_observer(handle_project_loading_result_system)
            .add_action(
                "phichain.save_project",
                save_project_system,
                Some(Hotkey::new(KeyCode::KeyS, vec![Modifier::Control])),
            )
            .add_action(
                "phichain.close_project",
                |mut events: EventWriter<UnloadProjectEvent>| {
                    events.write(UnloadProjectEvent);

                    Ok(())
                },
                Some(Hotkey::new(KeyCode::KeyW, vec![Modifier::Control])),
            )
            .add_heavy_action(
                "phichain.open_in_file_manager",
                |project: Res<Project>| {
                    let _ = open::that(project.path.0.clone());

                    Ok(())
                },
                None,
            );
    }
}

fn save_project_system(
    project: Res<Project>,
    mut toasts: ResMut<ToastsStorage>,
    mut history: ResMut<EditorHistory>,

    chart_params: SerializeChartParam,
    line_params: SerializeLineParam,
) -> Result {
    let result: anyhow::Result<()> = {
        let chart = serialize_chart(chart_params, line_params);
        let chart_string = serde_json::to_string(&chart)?;
        std::fs::write(project.path.chart_path(), chart_string)?;
        std::fs::write(
            project.path.meta_path(),
            serde_json::to_string(&project.meta).unwrap(),
        )?;

        Ok(())
    };

    match result {
        Ok(_) => {
            toasts.success(t!("project.save.succeed"));
            history.0.set_saved();
        }
        Err(error) => {
            toasts.error(t!("project.save.failed", error = error));
        }
    }

    Ok(())
}

#[derive(Event, Debug)]
pub struct LoadProjectEvent(pub PathBuf);

/// Load a project into the editor
///
/// This function will use [`phichain_game::load_project`] to load core game components and resources to the world
///
/// After [`phichain_game::load_project`] is executed successfully, this function will load editor-specific resources:
///
/// - [crate::selection::SelectedLine] will be inserted into the world
/// - Audio playback resources ([phichain_game::audio::InstanceHandle], [phichain_game::audio::AudioDuration] and [phichain_game::audio::AudioAssetId]) will be inserted into the world
/// - After all resources and entities above are added, [Project] will be inserted into the world,
///   indicating the editor is now in editing mode: all systems with run condition [`project_loaded`] will start working
fn load_project_system(
    mut commands: Commands,
    mut events: EventReader<LoadProjectEvent>,
    mut toasts: ResMut<ToastsStorage>,
) {
    if events.len() > 1 {
        warn!("Multiple projects are requested, ignoring previous ones");
    }

    if let Some(event) = events.read().last() {
        match Project::open(event.0.clone()) {
            Ok(project) => {
                phichain_game::loader::nonblocking::load_project(&project, &mut commands);
                // results will be handled in `handle_project_loading_result_system`
            }
            Err(error) => {
                let message = match error {
                    OpenProjectError::MissingFile(file) => {
                        t!("error.open_project.missing_file", file = file)
                    }
                    OpenProjectError::CannotOpenMeta(error) => {
                        t!("error.open_project.cannot_open_meta", error = error)
                    }
                    OpenProjectError::InvalidMeta(error) => {
                        t!("error.open_project.invalid_meta", error = error)
                    }
                };
                toasts.error(format!("{}: {}", t!("error.open_project.label"), message));
            }
        }
    }

    events.clear();
}

fn handle_project_loading_result_system(
    trigger: Trigger<ProjectLoadingResult>,

    mut commands: Commands,
    mut recent_projects: ResMut<Persistent<RecentProjects>>,
    mut telemetry: EventWriter<PushTelemetryEvent>,
    mut toasts: ResMut<ToastsStorage>,
) {
    let data = trigger.event();

    match &data.0 {
        Ok(data) => {
            telemetry.write(PushTelemetryEvent::new(
                "phichain.editor.project.loaded",
                json!({ "duration": data.duration.as_millis() }),
            ));
            recent_projects.push(RecentProject::new(
                data.project.meta.name.clone(),
                data.project.path.0.clone(),
            ));

            commands.queue(|world: &mut World| {
                let mut query = world.query_filtered::<Entity, With<Line>>();
                if let Some(first) = query.iter(world).next() {
                    world.insert_resource(crate::selection::SelectedLine(first));
                }

                // generate spectrogram
                let audio_asset_id = world.resource::<phichain_game::audio::AudioAssetId>().0;
                let audio_assets = world.resource::<Assets<AudioSource>>();
                let source = audio_assets
                    .get(audio_asset_id)
                    .expect("Expected audio loaded in Assets<AudioSource>");
                world.insert_resource(Spectrogram(spectrogram::make_spectrogram(source)));
            });

            commands.insert_resource(data.project.clone());
        }
        Err(error) => {
            let message = match error {
                LoadProjectError::CannotOpenChart(error) => {
                    t!("error.load_project.cannot_open_chart", error = error)
                }
                LoadProjectError::InvalidChart(error) => {
                    t!("error.load_project.invalid_chart", error = error)
                }
                LoadProjectError::MigrationFailed(error) => {
                    t!("error.load_project.migration_failed", error = error)
                }
                LoadProjectError::CannotLoadAudio(error) => {
                    let reason = match error {
                        LoadAudioError::UnknownFormat => t!("error.load_audio.unknown_format"),
                        LoadAudioError::UnsupportedFormat(format) => {
                            t!("error.load_audio.unsupported_format", format = format)
                        }
                        LoadAudioError::Io(io_error) => {
                            t!("error.load_audio.io", error = io_error)
                        }
                        LoadAudioError::Load(load_error) => {
                            t!("error.load_audio.load", error = load_error)
                        }
                    };

                    t!("error.load_project.cannot_load_audio", error = reason)
                }
            };

            toasts.error(message);
            telemetry.write(PushTelemetryEvent::new(
                "phichain.editor.project.load.failed",
                json!({}),
            ));
        }
    }
}

#[derive(Event, Debug)]
pub struct UnloadProjectEvent;

/// Unload a project from the editor
fn unload_project_system(
    world: &mut World,
    params: &mut SystemState<EventReader<UnloadProjectEvent>>,
) {
    let mut events = params.get_mut(world);
    if !events.is_empty() {
        events.clear();

        // remove the project first to stop all systems
        world.remove_resource::<Project>();

        // unload audio
        use phichain_game::audio::{AudioAssetId, AudioDuration, InstanceHandle};
        world.remove_resource::<InstanceHandle>();
        world.remove_resource::<AudioDuration>();
        let audio_asset_id = world.resource::<AudioAssetId>().0;
        let mut audios = world.resource_mut::<Assets<AudioSource>>();
        audios.remove(audio_asset_id);
        world.remove_resource::<AudioAssetId>();
        let audio = world.resource::<Audio>();
        audio.stop();

        // unload spectrogram resource
        use crate::spectrogram::Spectrogram;
        world.remove_resource::<Spectrogram>();

        // unload illustration
        use phichain_game::illustration::{Illustration, IllustrationAssetId};
        let mut illustration_query = world.query_filtered::<Entity, With<Illustration>>();
        let entities = illustration_query.iter(world).collect::<Vec<_>>();
        for entity in entities {
            world.entity_mut(entity).despawn();
        }
        if let Some(illustration_asset_id) =
            world.get_resource::<IllustrationAssetId>().map(|x| x.0)
        {
            let mut images = world.resource_mut::<Assets<Image>>();
            images.remove(illustration_asset_id);
        }

        // unload chart basic components
        use crate::selection::SelectedLine;
        use phichain_chart::{bpm_list::BpmList, offset::Offset};
        world.remove_resource::<Offset>();
        world.remove_resource::<BpmList>();
        world.remove_resource::<SelectedLine>();

        // unload lines, notes and events
        use phichain_chart::line::Line;
        let mut line_query = world.query_filtered::<Entity, (With<Line>, Without<ChildOf>)>();
        let entities = line_query.iter(world).collect::<Vec<_>>();
        for entity in entities {
            // notes and events will be despawned as children
            world.entity_mut(entity).despawn();
        }

        // despawn ghost entities created when despawning an entity with `keep_entity`
        let to_remove = world
            .query::<Entity>()
            .iter(world)
            .filter(|entity| {
                world
                    .inspect_entity(*entity)
                    .is_ok_and(|x| x.collect::<Vec<_>>().is_empty())
            })
            .collect::<Vec<_>>();
        for entity in to_remove {
            world.entity_mut(entity).despawn();
        }

        // clear editor history
        world.resource_mut::<EditorHistory>().0.clear();

        // reset editor timing
        use crate::timing::{ChartTime, Timing};
        world.resource_mut::<ChartTime>().0 = 0.0;
        world.resource_mut::<Timing>().seek_to(0.0);
    }
}

/// Create a new empty project
pub fn create_project(
    root_path: PathBuf,
    music_path: PathBuf,
    illustration_path: Option<PathBuf>,
    project_meta: ProjectMeta,
) -> anyhow::Result<()> {
    let project_path = ProjectPath(root_path);

    let mut target_music_path = project_path.sub_path("music");
    if let Some(ext) = music_path.extension() {
        target_music_path.set_extension(ext);
    }

    std::fs::copy(music_path, target_music_path).context("Failed to copy music file")?;

    if let Some(illustration_path) = illustration_path {
        let mut target_illustration_path = project_path.sub_path("illustration");
        if let Some(ext) = illustration_path.extension() {
            target_illustration_path.set_extension(ext);
        }

        std::fs::copy(illustration_path, target_illustration_path)
            .context("Failed to copy illustration file")?;
    }

    let meta_string = serde_json::to_string_pretty(&project_meta)?;
    std::fs::write(project_path.meta_path(), meta_string).context("Failed to write meta")?;

    let chart_string = serde_json::to_string_pretty(&PhichainChart::default())?;
    std::fs::write(project_path.chart_path(), chart_string).context("Failed to write chart")?;

    Ok(())
}
