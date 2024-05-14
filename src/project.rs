use anyhow::{bail, Context};
use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use serde::{Deserialize, Serialize};

use std::{fs::File, path::PathBuf};

use crate::{
    constants::ILLUSTRATION_BLUR,
    loader::{phichain::PhiChainLoader, Loader},
    notification::{ToastsExt, ToastsStorage},
    serialzation::PhiChainChart,
    tab::game::illustration::SpawnIllustrationEvent,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

    pub fn illustration_path(&self) -> PathBuf {
        self.0.join("illustration.png")
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
        if !self.illustration_path().is_file() {
            bail!("illustration.png is missing");
        }
        if !self.meta_path().is_file() {
            bail!("meta.json is missing");
        }

        let meta_file = File::open(self.meta_path()).context("Failed to open meta file")?;
        let meta: ProjectMeta = serde_json::from_reader(meta_file).context("Invalid meta file")?;

        let chart_file = File::open(self.chart_path()).context("Failed to open chart file")?;
        // just do validation here
        let _: PhiChainChart = serde_json::from_reader(chart_file).context("Invalid chart")?;

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

pub struct ProjectPlugin;

impl Plugin for ProjectPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadProjectEvent>()
            .add_systems(Update, load_project_system);
    }
}

#[derive(Event, Debug)]
pub struct LoadProjectEvent(pub PathBuf);

fn load_project_system(
    mut commands: Commands,
    mut events: EventReader<LoadProjectEvent>,
    mut toasts: ResMut<ToastsStorage>,
) {
    if events.len() > 1 {
        warn!("Mutiple projects are requested, ignoring previous ones");
    }

    if let Some(event) = events.read().last() {
        match Project::load(event.0.clone()) {
            Ok(project) => {
                let file = File::open(project.root_dir.join("chart.json")).unwrap();
                let illustraion_path = project.root_dir.join("illustration.png");
                PhiChainLoader::load(file, &mut commands);
                commands.add(|world: &mut World| {
                    // TODO: error handling
                    // TODO: move image loading to illsturation.rs, SpawnIllustrationEvent(PathBuf)
                    let mut images = world.resource_mut::<Assets<Image>>();
                    let image = image::open(illustraion_path)
                        .unwrap()
                        .blur(ILLUSTRATION_BLUR);
                    let rgb8 = image.as_rgba8().unwrap();
                    let handle = images.add(Image::new(
                        Extent3d {
                            width: image.width(),
                            height: image.height(),
                            depth_or_array_layers: 1,
                        },
                        TextureDimension::D2,
                        rgb8.clone().into_vec(),
                        TextureFormat::Rgba8UnormSrgb,
                        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
                    ));
                    world.send_event(SpawnIllustrationEvent(handle));
                });
                commands.insert_resource(project);
            }
            Err(error) => {
                toasts.error(format!("Failed to open project: {:?}", error));
            }
        }
    }

    events.clear();
}

pub fn create_project(
    root_path: PathBuf,
    music_path: PathBuf,
    illustration_path: PathBuf,
    project_meta: ProjectMeta,
) -> anyhow::Result<()> {
    let project_path = ProjectPath(root_path);
    std::fs::copy(music_path, project_path.music_path()).context("Failed to copy music file")?;
    std::fs::copy(illustration_path, project_path.illustration_path())
        .context("Failed to copy illustration file")?;
    let meta_string = serde_json::to_string_pretty(&project_meta).unwrap();
    std::fs::write(project_path.meta_path(), meta_string).context("Failed to write meta")?;
    let chart_string = serde_json::to_string_pretty(&PhiChainChart::default()).unwrap();
    std::fs::write(project_path.chart_path(), chart_string).context("Failed to write meta")?;

    Ok(())
}
