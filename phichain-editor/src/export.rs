use crate::file::{PickingEvent, PickingKind};
use crate::notification::{ToastsExt, ToastsStorage};
use crate::project::{project_loaded, Project};
use anyhow::{bail, Context};
use bevy::app::App;
use bevy::prelude::*;
use phichain_chart::format::official::OfficialChart;
use phichain_chart::format::Format;
use phichain_chart::serialization::PhiChainChart;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;

pub struct ExportPlugin;

impl Plugin for ExportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, export_official_system.run_if(project_loaded()));
    }
}

fn export_official(path: &Path, project: &Project) -> anyhow::Result<()> {
    let zip_path = path.join("chart.zip");
    if zip_path.exists() {
        bail!("chart.zip already exists in the folder");
    }

    let file = fs::File::create(zip_path)?;

    let mut zip = zip::ZipWriter::new(file);

    zip.start_file("chart.json", SimpleFileOptions::default())?;
    let chart_file = fs::File::open(project.path.chart_path())?;
    let chart: PhiChainChart = serde_json::from_reader(chart_file)?;
    let official = OfficialChart::from_phichain(chart)?;
    zip.write_all(serde_json::to_string(&official)?.as_bytes())?;

    if let Some(illustration_path) = project.path.illustration_path() {
        let filename = illustration_path
            .file_name()
            .context("Failed to get filename of illustration")?
            .to_str()
            .context("Failed to convert illustration filename to str")?;
        zip.start_file(filename, SimpleFileOptions::default())?;
        let mut illustration_file = fs::File::open(illustration_path)?;
        let mut illustration_data = Vec::new();
        illustration_file.read_to_end(&mut illustration_data)?;
        zip.write_all(&illustration_data)?;
    }

    if let Some(music_path) = project.path.music_path() {
        let filename = music_path
            .file_name()
            .context("Failed to get filename of music")?
            .to_str()
            .context("Failed to convert music filename to str")?;
        zip.start_file(filename, SimpleFileOptions::default())?;
        let mut music_file = fs::File::open(music_path)?;
        let mut music_data = Vec::new();
        music_file.read_to_end(&mut music_data)?;
        zip.write_all(&music_data)?;
    }

    zip.start_file("info.txt", SimpleFileOptions::default())?;
    zip.write_all(
        format!(
            "#
Name: {}
Level: {}
Composer: {}
Illustrator: {}
Charter: {}
",
            project.meta.name,
            project.meta.level,
            project.meta.composer,
            project.meta.illustrator,
            project.meta.charter
        )
        .as_bytes(),
    )?;

    zip.finish()?;

    Ok(())
}

fn export_official_system(
    mut event_reader: EventReader<PickingEvent>,
    project: Res<Project>,
    mut toasts: ResMut<ToastsStorage>,
) {
    for PickingEvent { path, kind } in event_reader.read() {
        if !matches!(kind, PickingKind::ExportOfficial) {
            continue;
        }

        let Some(path) = path else {
            return;
        };

        match export_official(path, &project) {
            Ok(_) => {
                toasts.success("Successfully exported official chart");
            }
            Err(error) => {
                toasts.error(format!("Failed to export official chart: {}", error));
            }
        }
    }
}
