use anyhow::{Context, Result};
use bevy::prelude::*;
use chrono::{DateTime, Local};
use phichain_chart::serialization::PhichainChart;
use std::fs;
use std::path::{Path, PathBuf};

pub struct BackupManager {
    autosave_dir: PathBuf,
}

impl BackupManager {
    pub fn new(project_root: &Path) -> Self {
        let autosave_dir = project_root.join(".autosave");
        Self { autosave_dir }
    }

    pub fn create_backup(&self, chart: &PhichainChart) -> Result<PathBuf> {
        fs::create_dir_all(&self.autosave_dir).context("Failed to create autosave directory")?;

        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("chart_autosave_{}.json", timestamp);
        let backup_path = self.autosave_dir.join(backup_filename);

        let chart_string =
            serde_json::to_string(chart).context("Failed to serialize chart for auto-save")?;

        fs::write(&backup_path, chart_string).context("Failed to write auto-save backup file")?;

        info!("Auto-save backup created: {:?}", backup_path);
        Ok(backup_path)
    }

    pub fn cleanup_old_backups(&self, max_count: usize) -> Result<()> {
        if !self.autosave_dir.exists() {
            return Ok(());
        }

        let mut backup_files = self.list_backup_files()?;

        if backup_files.len() <= max_count {
            return Ok(());
        }

        // sort by creation time (newest first)
        backup_files.sort_by(|a, b| {
            let time_a = fs::metadata(a)
                .and_then(|m| m.created())
                .unwrap_or(std::time::UNIX_EPOCH);
            let time_b = fs::metadata(b)
                .and_then(|m| m.created())
                .unwrap_or(std::time::UNIX_EPOCH);
            time_b.cmp(&time_a)
        });

        // remove excess files
        for file_to_remove in backup_files.iter().skip(max_count) {
            if let Err(e) = fs::remove_file(file_to_remove) {
                warn!(
                    "Failed to remove old backup file {:?}: {}",
                    file_to_remove, e
                );
            } else {
                info!("Removed old backup file: {:?}", file_to_remove);
            }
        }

        Ok(())
    }

    fn list_backup_files(&self) -> Result<Vec<PathBuf>> {
        let mut backup_files = Vec::new();

        if !self.autosave_dir.exists() {
            return Ok(backup_files);
        }

        let entries =
            fs::read_dir(&self.autosave_dir).context("Failed to read autosave directory")?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with("chart_autosave_") && filename.ends_with(".json") {
                        backup_files.push(path);
                    }
                }
            }
        }

        Ok(backup_files)
    }

    pub fn get_latest_backup(&self) -> Result<Option<(PathBuf, DateTime<Local>)>> {
        let backup_files = self.list_backup_files()?;

        if backup_files.is_empty() {
            return Ok(None);
        }

        let mut latest_file: Option<(PathBuf, std::time::SystemTime)> = None;

        for file in backup_files {
            let metadata = fs::metadata(&file).context("Failed to read backup file metadata")?;
            let created = metadata
                .created()
                .context("Failed to get backup file creation time")?;

            if latest_file.is_none() || created > latest_file.as_ref().unwrap().1 {
                latest_file = Some((file, created));
            }
        }

        if let Some((path, system_time)) = latest_file {
            let datetime: DateTime<Local> = system_time.into();
            Ok(Some((path, datetime)))
        } else {
            Ok(None)
        }
    }
}
