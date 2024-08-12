use anyhow::{anyhow, bail, Context};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::action::ActionRegistrationExt;
use crate::audio::load_audio;
use crate::editing::history::EditorHistory;
use crate::exporter::phichain::PhichainExporter;
use crate::exporter::Exporter;
use crate::notification::{ToastsExt, ToastsStorage};
use crate::recent_projects::{PersistentRecentProjectsExt, RecentProject, RecentProjects};
use bevy::ecs::system::SystemState;
use bevy_kira_audio::{Audio, AudioControl, AudioSource};
use bevy_persistent::Persistent;
use phichain_chart::line::Line;
use phichain_chart::serialization::PhichainChart;
use phichain_game::illustration::load_illustration;
use std::path::Path;
use std::{fs::File, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProjectMeta {
    pub composer: String,
    pub charter: String,
    pub illustrator: String,
    pub name: String,
    pub level: String,
}

#[derive(Resource)]
pub struct Project {
    pub path: ProjectPath,
    pub meta: ProjectMeta,
}

impl Project {
    pub fn load(root_dir: PathBuf) -> anyhow::Result<Self> {
        ProjectPath(root_dir).into_project()
    }
}

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
                    && path.extension().map_or(false, |ext| {
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

        Ok(Project { path: self, meta })
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

pub struct ProjectPlugin;

impl Plugin for ProjectPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadProjectEvent>()
            .add_systems(Update, load_project_system.run_if(project_not_loaded()))
            .add_event::<UnloadProjectEvent>()
            .add_systems(PreUpdate, unload_project_system.run_if(project_loaded()))
            .register_action("phichain.project.save", save_project_system)
            .register_action(
                "phichain.project.unload",
                |mut events: EventWriter<UnloadProjectEvent>| {
                    events.send(UnloadProjectEvent);
                },
            );
    }
}

fn save_project_system(world: &mut World) {
    world.resource_scope(|world, mut history: Mut<EditorHistory>| {
        if let Ok(chart) = PhichainExporter::export(world) {
            let project = world.resource::<Project>();
            let chart_result = std::fs::write(project.path.chart_path(), chart);
            let meta_result = std::fs::write(
                project.path.meta_path(),
                serde_json::to_string(&project.meta).unwrap(),
            );

            let mut toasts = world.resource_mut::<ToastsStorage>();
            match chart_result.and(meta_result) {
                Ok(_) => {
                    toasts.success(t!("project.save.succeed"));
                    history.0.set_saved(true);
                }
                Err(error) => {
                    toasts.error(t!("project.save.failed", error = error));
                }
            }
        }
    });
}

#[derive(Event, Debug)]
pub struct LoadProjectEvent(pub PathBuf);

/// Load a project into the editor
///
/// # Resources and entities involved when loading projects
///
/// - [InstanceHandle], [AudioDuration] and [AudioAssetId] will be inserted into the world
/// - A entity with component [Illustration] will be spawned into the world, [IllustrationAssetId] will be inserted into the world
///
/// ---
///
/// - [crate::audio::Offset] will be inserted into the world
/// - [crate::timing::BpmList] will be inserted into the world
/// - [crate::selection::SelectedLine] will be inserted into the world
/// - Entities with components [phichain_chart::line::LineBundle] and [phichain_chart::note::NoteBundle] will be spawned into the world, with parent-child relationship
///
/// ---
///
/// - After all resources and entities above are added, [Project] will be inserted into the world,
///   indicating the editor is now in editing mode: all systems with run condition [`project_loaded`] will start working
fn load_project_system(
    mut commands: Commands,
    mut events: EventReader<LoadProjectEvent>,
    mut toasts: ResMut<ToastsStorage>,

    mut recent_projects: ResMut<Persistent<RecentProjects>>,
) {
    if events.len() > 1 {
        warn!("Multiple projects are requested, ignoring previous ones");
    }

    if let Some(event) = events.read().last() {
        match Project::load(event.0.clone()) {
            Ok(project) => {
                let file = File::open(project.path.chart_path()).unwrap();
                if let Err(error) = phichain_game::load(file, &mut commands) {
                    toasts.error(format!("Failed to load chart: {:?}", error));
                } else {
                    recent_projects.push(RecentProject::new(
                        project.meta.name.clone(),
                        project.path.0.clone(),
                    ));

                    commands.add(|world: &mut World| {
                        let mut query = world.query_filtered::<Entity, With<Line>>();
                        if let Some(first) = query.iter(world).next() {
                            world.insert_resource(crate::selection::SelectedLine(first));
                        }
                    });

                    if let Some(illustration_path) = project.path.illustration_path() {
                        load_illustration(illustration_path, &mut commands);
                    }

                    // unwrap: if Project::load is ok, music_path() must return Some
                    let audio_path = project.path.music_path().unwrap();
                    load_audio(audio_path, &mut commands);
                    commands.insert_resource(project);
                }
            }
            Err(error) => {
                toasts.error(format!("Failed to open project: {:?}", error));
            }
        }
    }

    events.clear();
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
        use crate::audio::{AudioAssetId, AudioDuration, InstanceHandle};
        world.remove_resource::<InstanceHandle>();
        world.remove_resource::<AudioDuration>();
        let audio_asset_id = world.resource::<AudioAssetId>().0;
        let mut audios = world.resource_mut::<Assets<AudioSource>>();
        audios.remove(audio_asset_id);
        world.remove_resource::<AudioAssetId>();
        let audio = world.resource::<Audio>();
        audio.stop();

        // unload illustration
        use phichain_game::illustration::{Illustration, IllustrationAssetId};
        let mut illustration_query = world.query_filtered::<Entity, With<Illustration>>();
        let entities = illustration_query.iter(world).collect::<Vec<_>>();
        for entity in entities {
            world.entity_mut(entity).despawn_recursive();
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
        let mut line_query = world.query_filtered::<Entity, With<Line>>();
        let entities = line_query.iter(world).collect::<Vec<_>>();
        for entity in entities {
            // notes and events will be despawned as children
            world.entity_mut(entity).despawn_recursive();
        }
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

    let meta_string = serde_json::to_string_pretty(&project_meta).unwrap();
    std::fs::write(project_path.meta_path(), meta_string).context("Failed to write meta")?;

    let chart_string = serde_json::to_string_pretty(&PhichainChart::default()).unwrap();
    std::fs::write(project_path.chart_path(), chart_string).context("Failed to write chart")?;

    Ok(())
}
