use crate::action::ActionRegistrationExt;
use crate::file::{pick_folder, PickingEvent, PickingKind};
use crate::hotkey::modifier::Modifier;
use crate::hotkey::Hotkey;
use crate::notification::{ToastsExt, ToastsStorage};
use crate::project::{project_loaded, Project};
use anyhow::Context;
use bevy::app::App;
use bevy::prelude::*;
use phichain_chart::serialization::PhichainChart;
use phichain_format::official::OfficialChart;
use phichain_format::official::OfficialOutputOptions;
use phichain_format::rpe::RpeChart;
use phichain_format::ChartFormat;
use rfd::FileDialog;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;

pub struct ExportPlugin;

impl Plugin for ExportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, export_official_system.run_if(project_loaded()))
            .add_systems(Update, export_rpe_system.run_if(project_loaded()))
            .add_heavy_action(
                "phichain.export_as_official",
                export_as_official_system,
                Some(Hotkey::new(
                    KeyCode::KeyO,
                    vec![Modifier::Control, Modifier::Shift],
                )),
            )
            .add_heavy_action("phichain.export_as_rpe", export_as_rpe_system, None);
    }
}

fn export_as_official_system(world: &mut World) -> Result {
    pick_folder(world, PickingKind::ExportOfficial, FileDialog::new());

    Ok(())
}

fn export_as_rpe_system(world: &mut World) -> Result {
    pick_folder(world, PickingKind::ExportRpe, FileDialog::new());

    Ok(())
}

/// Generates the export path under a path, ensuring the path does not already exist
fn get_export_path(path: &Path, index: usize) -> Option<PathBuf> {
    if index >= 10 {
        None
    } else {
        let zip_path = path.join(if index == 0 {
            "chart.zip".to_string()
        } else {
            format!("chart({index}).zip")
        });

        if zip_path.exists() {
            get_export_path(path, index + 1)
        } else {
            Some(zip_path)
        }
    }
}

fn export(path: &Path, project: &Project, chart_string: &str) -> anyhow::Result<PathBuf> {
    let zip_path = get_export_path(path, 0).context("Failed to get export path")?;

    let file = fs::File::create(&zip_path)?;

    let mut zip = zip::ZipWriter::new(file);

    zip.start_file("chart.json", SimpleFileOptions::default())?;
    zip.write_all(chart_string.as_bytes())?;

    let mut info_txt = format!(
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
    );

    if let Some(illustration_path) = project.path.illustration_path() {
        let filename = illustration_path
            .file_name()
            .context("Failed to get filename of illustration")?
            .to_str()
            .context("Failed to convert illustration filename to str")?;
        info_txt.push_str(format!("Picture: {}\n", filename).as_str());

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
        info_txt.push_str(format!("Song: {}\n", filename).as_str());

        zip.start_file(filename, SimpleFileOptions::default())?;
        let mut music_file = fs::File::open(music_path)?;
        let mut music_data = Vec::new();
        music_file.read_to_end(&mut music_data)?;
        zip.write_all(&music_data)?;
    }

    zip.start_file("info.txt", SimpleFileOptions::default())?;

    zip.write_all(info_txt.as_bytes())?;

    zip.finish()?;

    Ok(zip_path)
}

fn export_official(path: &Path, project: &Project) -> anyhow::Result<PathBuf> {
    let chart_file = fs::File::open(project.path.chart_path())?;
    let chart: PhichainChart = serde_json::from_reader(chart_file)?;
    let official = OfficialChart::from_phichain(chart, &OfficialOutputOptions::default())?;

    export(path, project, &serde_json::to_string(&official)?)
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
            Ok(path) => {
                toasts.success(t!("export.official.success", path = path.to_string_lossy()));
            }
            Err(error) => {
                toasts.error(t!("export.official.failed", error = error));
            }
        }
    }
}

fn export_rpe(path: &Path, project: &Project) -> anyhow::Result<PathBuf> {
    let chart_file = fs::File::open(project.path.chart_path())?;
    let chart: PhichainChart = serde_json::from_reader(chart_file)?;
    let rpe = RpeChart::from_phichain(chart, &())?;

    export(path, project, &serde_json::to_string(&rpe)?)
}

fn export_rpe_system(
    mut event_reader: EventReader<PickingEvent>,
    project: Res<Project>,
    mut toasts: ResMut<ToastsStorage>,
) {
    for PickingEvent { path, kind } in event_reader.read() {
        if !matches!(kind, PickingKind::ExportRpe) {
            continue;
        }

        let Some(path) = path else {
            return;
        };

        match export_rpe(path, &project) {
            Ok(path) => {
                toasts.success(t!("export.rpe.success", path = path.to_string_lossy()));
            }
            Err(error) => {
                toasts.error(t!("export.rpe.failed", error = error));
            }
        }
    }
}
