use bevy::prelude::*;
use bevy_persistent::Persistent;
use undo::At;

mod backup;

use crate::editing::history::EditorHistory;
use crate::notification::{ToastsExt, ToastsStorage};
use crate::project::project_loaded;
use crate::settings::EditorSettings;
pub use backup::BackupManager;
use phichain_chart::project::Project;

pub struct AutoSavePlugin;

impl Plugin for AutoSavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastEditTime>()
            .init_resource::<AutoSaveState>()
            .add_systems(Update, track_edit_time_system.run_if(project_loaded()))
            .add_systems(Update, auto_save_system.run_if(project_loaded()));
    }
}

#[derive(Resource, Default)]
pub struct LastEditTime {
    time: Option<f32>,
}

impl LastEditTime {
    pub fn update(&mut self, time: f32) {
        self.time = Some(time);
    }

    pub fn elapsed(&self, current_time: f32) -> f32 {
        match self.time {
            Some(time) => current_time - time,
            None => f32::INFINITY, // Never trigger idle save if no edit has happened
        }
    }
}

#[derive(Resource, Default)]
pub struct AutoSaveState {
    pub last_save_time: Option<f32>,
    pub last_auto_saved_head: Option<At>,
    pub is_saving: bool,
}

fn auto_save_system(world: &mut World, mut last_triggered_time: Local<Option<f32>>) {
    let current_time = world.resource::<Time>().elapsed_secs();

    let settings = world.resource::<Persistent<EditorSettings>>().autosave;

    if !settings.enabled {
        return;
    }

    if current_time - last_triggered_time.unwrap_or_default() < settings.interval_secs {
        return;
    }

    last_triggered_time.replace(current_time);

    debug!("Triggering autosave...");

    let state = world.resource::<AutoSaveState>();
    let history = world.resource::<EditorHistory>();
    let last_edit = world.resource::<LastEditTime>();

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
        last_edit.elapsed(current_time) >= settings.idle_delay_secs && last_edit.time.is_some();

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
            .and_then(|_| backup_manager.cleanup_old_backups(settings.max_backup_count))
    };

    match result {
        Ok(_) => {
            let mut state = world.resource_mut::<AutoSaveState>();
            state.last_save_time = Some(current_time);
            state.last_auto_saved_head = Some(current_head);
            state.is_saving = false;

            world
                .resource_mut::<ToastsStorage>()
                .info(t!("project.autosave.succeed"));

            info!("Auto-save completed successfully");
        }
        Err(e) => {
            world.resource_mut::<AutoSaveState>().is_saving = false;

            world
                .resource_mut::<ToastsStorage>()
                .error(t!("project.autosave.failed", error = e));

            warn!("Auto-save failed: {}", e);
        }
    }
}

fn track_edit_time_system(
    time: Res<Time>,
    mut last_edit: ResMut<LastEditTime>,
    history: Res<EditorHistory>,
) {
    if history.is_changed() && !history.0.is_saved() {
        last_edit.update(time.elapsed_secs());
    }
}
