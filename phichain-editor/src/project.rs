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
            .add_observer(project_loading_result_observer)
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

fn project_loading_result_observer(
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// Create a minimal WAV file for testing
    fn create_dummy_wav(path: &std::path::Path) {
        let mut file = std::fs::File::create(path).unwrap();
        let data_size: u32 = 0;
        let file_size: u32 = 36 + data_size;
        file.write_all(b"RIFF").unwrap();
        file.write_all(&file_size.to_le_bytes()).unwrap();
        file.write_all(b"WAVE").unwrap();
        file.write_all(b"fmt ").unwrap();
        file.write_all(&16u32.to_le_bytes()).unwrap();
        file.write_all(&1u16.to_le_bytes()).unwrap(); // PCM
        file.write_all(&1u16.to_le_bytes()).unwrap(); // mono
        file.write_all(&44100u32.to_le_bytes()).unwrap();
        file.write_all(&88200u32.to_le_bytes()).unwrap();
        file.write_all(&2u16.to_le_bytes()).unwrap();
        file.write_all(&16u16.to_le_bytes()).unwrap();
        file.write_all(b"data").unwrap();
        file.write_all(&data_size.to_le_bytes()).unwrap();
    }

    /// Create a dummy PNG file for testing
    fn create_dummy_png(path: &std::path::Path) {
        let data: &[u8] = &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
            0x00, 0x90, 0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
            0x42, 0x60, 0x82,
        ];
        std::fs::write(path, data).unwrap();
    }

    fn sample_meta() -> ProjectMeta {
        ProjectMeta {
            name: "Test Song".to_string(),
            composer: "Test Composer".to_string(),
            charter: "Test Charter".to_string(),
            illustrator: "Test Illustrator".to_string(),
            level: "IN Lv.15".to_string(),
        }
    }

    #[test]
    fn test_create_project_with_music_only() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path().join("my_project");
        std::fs::create_dir(&project_dir).unwrap();

        let music_file = tmp.path().join("song.wav");
        create_dummy_wav(&music_file);

        let meta = sample_meta();
        create_project(project_dir.clone(), music_file, None, meta.clone()).unwrap();

        // Verify music was copied
        assert!(project_dir.join("music.wav").exists());

        // Verify meta.json was created with correct content
        let saved_meta: ProjectMeta =
            serde_json::from_str(&std::fs::read_to_string(project_dir.join("meta.json")).unwrap())
                .unwrap();
        assert_eq!(saved_meta, meta);

        // Verify chart.json was created
        assert!(project_dir.join("chart.json").exists());
        let chart: PhichainChart =
            serde_json::from_str(&std::fs::read_to_string(project_dir.join("chart.json")).unwrap())
                .unwrap();
        assert_eq!(chart.lines.len(), 1);

        // Verify no illustration was created
        assert!(ProjectPath(project_dir).illustration_path().is_none());
    }

    #[test]
    fn test_create_project_with_illustration() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path().join("my_project");
        std::fs::create_dir(&project_dir).unwrap();

        let music_file = tmp.path().join("song.mp3");
        std::fs::write(&music_file, b"fake mp3").unwrap();

        let illustration_file = tmp.path().join("cover.png");
        create_dummy_png(&illustration_file);

        create_project(
            project_dir.clone(),
            music_file,
            Some(illustration_file),
            sample_meta(),
        )
        .unwrap();

        assert!(project_dir.join("music.mp3").exists());
        assert!(project_dir.join("illustration.png").exists());
    }

    #[test]
    fn test_create_and_open_project_round_trip() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path().join("my_project");
        std::fs::create_dir(&project_dir).unwrap();

        let music_file = tmp.path().join("song.wav");
        create_dummy_wav(&music_file);

        let meta = sample_meta();
        create_project(project_dir.clone(), music_file, None, meta.clone()).unwrap();

        // Open the project we just created
        let project = Project::open(project_dir).unwrap();

        assert_eq!(project.meta, meta);
        assert!(project.path.music_path().is_some());
        assert!(project.path.chart_path().is_file());
    }

    #[test]
    fn test_open_project_missing_music() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path().join("incomplete_project");
        std::fs::create_dir(&project_dir).unwrap();

        // Only create chart.json and meta.json, no music
        std::fs::write(
            project_dir.join("chart.json"),
            serde_json::to_string(&PhichainChart::default()).unwrap(),
        )
        .unwrap();
        std::fs::write(
            project_dir.join("meta.json"),
            serde_json::to_string(&sample_meta()).unwrap(),
        )
        .unwrap();

        let result = Project::open(project_dir);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            OpenProjectError::MissingFile(_)
        ));
    }

    #[test]
    fn test_open_project_missing_chart() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path().join("no_chart");
        std::fs::create_dir(&project_dir).unwrap();

        let music_file = project_dir.join("music.wav");
        create_dummy_wav(&music_file);
        std::fs::write(
            project_dir.join("meta.json"),
            serde_json::to_string(&sample_meta()).unwrap(),
        )
        .unwrap();

        let result = Project::open(project_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_project_missing_music_file() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path().join("my_project");
        std::fs::create_dir(&project_dir).unwrap();

        let nonexistent_music = tmp.path().join("nonexistent.wav");
        let result = create_project(project_dir, nonexistent_music, None, sample_meta());
        assert!(result.is_err());
    }
}
