use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;
use undo::At;

mod backup;

use crate::editing::history::EditorHistory;
use crate::project::project_loaded;
pub use backup::BackupManager;
use phichain_chart::project::Project;

const AUTO_SAVE_INTERVAL_SECS: f32 = 20.0;
const MAX_BACKUP_COUNT: usize = 5;
const IDLE_SAVE_DELAY_SECS: f32 = 3.0;

pub struct AutoSavePlugin;

impl Plugin for AutoSavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastEditTime>()
            .init_resource::<AutoSaveState>()
            .add_systems(Update, track_edit_time_system.run_if(project_loaded()))
            .add_systems(
                Update,
                auto_save_system.run_if(
                    on_timer(Duration::from_secs_f32(AUTO_SAVE_INTERVAL_SECS))
                        .and(project_loaded()),
                ),
            );
    }
}

#[derive(Resource, Default)]
pub struct LastEditTime {
    time: Option<f64>,
}

impl LastEditTime {
    pub fn update(&mut self, time: f64) {
        self.time = Some(time);
    }

    pub fn elapsed(&self, current_time: f64) -> f64 {
        match self.time {
            Some(time) => current_time - time,
            None => f64::INFINITY, // Never trigger idle save if no edit has happened
        }
    }
}

#[derive(Resource, Default)]
pub struct AutoSaveState {
    pub last_save_time: Option<f64>,
    pub last_auto_saved_head: Option<At>,
    pub is_saving: bool,
}

fn auto_save_system(world: &mut World) {
    debug!("Triggering autosave...");
    let current_time = world.resource::<Time>().elapsed_secs_f64();

    let history = world.resource::<EditorHistory>();
    let last_edit = world.resource::<LastEditTime>();
    let state = world.resource::<AutoSaveState>();

    if state.is_saving || history.0.is_saved() {
        debug!("Skipping auto-save: is saving or already saved");
        return;
    }

    let current_head = history.0.head();
    let has_new_edits = match &state.last_auto_saved_head {
        Some(last_head) => current_head != *last_head,
        None => true, // first auto-save
    };

    if !has_new_edits {
        debug!("Skipping auto-save: no new edits since last auto-save");
        return;
    }

    let idle =
        last_edit.elapsed(current_time) >= IDLE_SAVE_DELAY_SECS as f64 && last_edit.time.is_some();

    if !idle {
        debug!("Skipping auto-save: not idle");
        return;
    }

    let project = world.resource::<Project>();
    let project_path = project.path.0.clone();

    world.resource_mut::<AutoSaveState>().is_saving = true;

    let result = {
        let chart = phichain_game::serialization::serialize_chart(world);
        let backup_manager = BackupManager::new(&project_path);
        backup_manager
            .create_backup(&chart)
            .and_then(|_| backup_manager.cleanup_old_backups(MAX_BACKUP_COUNT))
    };

    let mut state = world.resource_mut::<AutoSaveState>();
    match result {
        Ok(_) => {
            state.last_save_time = Some(current_time);
            state.last_auto_saved_head = Some(current_head);
            info!("Auto-save completed successfully");
        }
        Err(e) => {
            warn!("Auto-save failed: {}", e);
        }
    }

    state.is_saving = false;
}

fn track_edit_time_system(
    time: Res<Time>,
    mut last_edit: ResMut<LastEditTime>,
    history: Res<EditorHistory>,
) {
    if history.is_changed() && !history.0.is_saved() {
        last_edit.update(time.elapsed_secs_f64());
    }
}
